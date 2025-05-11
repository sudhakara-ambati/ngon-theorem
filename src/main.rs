use fastrand;
use std::f64::consts::PI;
use std::time::Instant;
use std::borrow::Cow;
use geo::{Line, Point, Contains};
use geo::line_intersection::{line_intersection, LineIntersection};
use rust_xlsxwriter::{Workbook};
use wgpu::{self};
use wgpu::util::DeviceExt;
use bytemuck::{Pod, Zeroable};
use futures_intrusive::channel::shared::oneshot_channel;

fn generate_polygon_points(n: usize) -> Vec<(f64, f64)> {
    let angle_step = 2.0 * PI / n as f64;
    let rotation_offset = if n % 2 == 0 { -PI / 2.0 } else { -PI / 2.0 - angle_step / 2.0 };
    
    (0..n).map(|i| {
        let angle = i as f64 * angle_step + rotation_offset;
        (angle.cos(), angle.sin())
    }).collect()
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct SimulationParams {
    n: u32,
    iterations_per_thread: u32,
    seed: u32,
    _padding: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Point2D {
    x: f32,
    y: f32,
}

fn get_n_values_to_process() -> Vec<usize> {
    let mut values = Vec::new();
    for n in 3..=500 {
        values.push(n);
    }
    values
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
    
    let adapter = pollster::block_on(instance.request_adapter(
        &wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
        }
    )).expect("Failed to find an appropriate adapter");
    
    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("Polygon Monte Carlo Device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits {
                max_compute_workgroup_size_x: 1024,
                max_compute_invocations_per_workgroup: 1024,
                ..wgpu::Limits::default()
            },
            memory_hints: Default::default(),
            trace: Default::default(),
        },
    )).expect("Failed to create device");
    
    println!("Using GPU: {}", adapter.get_info().name);
    
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Monte Carlo Compute Shader"),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("monte_carlo.wgsl"))),
    });
    
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    
    worksheet.write_string(0, 0, "n").expect("Failed to write header");
    worksheet.write_string(0, 1, "probability").expect("Failed to write header");
    worksheet.write_string(0, 2, "std_dev").expect("Failed to write header");
    worksheet.write_string(0, 3, "ci_lower").expect("Failed to write header");
    worksheet.write_string(0, 4, "ci_upper").expect("Failed to write header");
    worksheet.write_string(0, 5, "time_seconds").expect("Failed to write header");
    
    let workgroup_size = 256;
    let num_workgroups = 4096;
    let total_threads = workgroup_size * num_workgroups;
    
    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Monte Carlo Pipeline"),
        layout: None,
        module: &shader,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    });
    
    let result_buffer_size = total_threads * std::mem::size_of::<u32>() as u32;
    let result_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Result Buffer"),
        size: result_buffer_size as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    
    let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Staging Buffer"),
        size: result_buffer_size as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    
    let mut row = 1;
    let n_values = get_n_values_to_process();
    
    for n in n_values {
        println!("Processing n = {}", n);
        let start_time = Instant::now();
        
        let points = generate_polygon_points(n);
        
        let mut gpu_points = Vec::with_capacity(n);
        for &(x, y) in &points {
            gpu_points.push(Point2D { x: x as f32, y: y as f32 });
        }
        
        let points_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Polygon Points Buffer"),
            contents: bytemuck::cast_slice(&gpu_points),
            usage: wgpu::BufferUsages::STORAGE,
        });
        
        let fixed_iterations: u32 = 1_000_000_000;
        let base_iterations: u32 = fixed_iterations / total_threads as u32;
        
        let params = SimulationParams {
            n: n as u32,
            iterations_per_thread: base_iterations,
            seed: fastrand::u32(..),
            _padding: 0,
        };
        
        let total_iterations = params.iterations_per_thread * total_threads as u32;
        
        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Parameters Buffer"),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        
        let bind_group_layout = compute_pipeline.get_bind_group_layout(0);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Monte Carlo Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: points_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: result_buffer.as_entire_binding(),
                },
            ],
        });
        
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Compute Encoder"),
        });
        
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Monte Carlo Compute Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&compute_pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            compute_pass.dispatch_workgroups(num_workgroups as u32, 1, 1);
        }
        
        encoder.copy_buffer_to_buffer(&result_buffer, 0, &staging_buffer, 0, result_buffer_size as u64);
        queue.submit(std::iter::once(encoder.finish()));
        
        let buffer_slice = staging_buffer.slice(..);
        
        let (tx, rx) = oneshot_channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });
        
        device.poll(wgpu::MaintainBase::Wait);
        
        match pollster::block_on(rx.receive()) {
            Some(Ok(result)) => {
                let data = buffer_slice.get_mapped_range();
                let results: Vec<u32> = bytemuck::cast_slice(&data).to_vec();
                drop(data);
                staging_buffer.unmap();
                
                let intersect_count: u64 = results.iter().map(|&x| x as u64).sum();
                let probability = intersect_count as f64 / total_iterations as f64;
                let std_dev = (probability * (1.0 - probability) / total_iterations as f64).sqrt();
                let z_score = 1.96;
                let ci_lower = (probability - z_score * std_dev).max(0.0);
                let ci_upper = (probability + z_score * std_dev).min(1.0);
                
                let elapsed = start_time.elapsed();
                let elapsed_seconds = elapsed.as_secs_f64();
                
                println!(
                    "n = {}, Probability = {:.6}, Std Dev = {:.6}, 95% CI = [{:.6}, {:.6}], Time: {:.2}s", 
                    n, probability, std_dev, ci_lower, ci_upper, elapsed_seconds
                );
                
                worksheet.write_number(row, 0, n as f64).expect("Failed to write n");
                worksheet.write_number(row, 1, probability).expect("Failed to write probability");
                worksheet.write_number(row, 2, std_dev).expect("Failed to write std_dev");
                worksheet.write_number(row, 3, ci_lower).expect("Failed to write ci_lower");
                worksheet.write_number(row, 4, ci_upper).expect("Failed to write ci_upper");
                worksheet.write_number(row, 5, elapsed_seconds).expect("Failed to write time");
                
                row += 1;
            },
            _ => println!("Failed to map buffer for n = {}", n),
        }
    }
    
    worksheet.autofit();
    workbook.save("polygon_intersection_results_gpu.xlsx").expect("Failed to save Excel file");
    
    Ok(())
}