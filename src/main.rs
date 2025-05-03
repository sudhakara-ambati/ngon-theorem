use fastrand;
use std::f64::consts::PI;
use std::time::Instant;
use geo::{Line, Polygon, Point, Contains};
use geo::line_intersection::{line_intersection, LineIntersection};
use rayon::prelude::*;
use rust_xlsxwriter::{Workbook};

fn generate_polygon_points(n: usize) -> Vec<(f64, f64)> {
    let angle_step = 2.0 * PI / n as f64;
    let rotation_offset = if n % 2 == 0 { -PI / 2.0 } else { -PI / 2.0 - angle_step / 2.0 };
    
    (0..n).map(|i| {
        let angle = i as f64 * angle_step + rotation_offset;
        (angle.cos(), angle.sin())
    }).collect()
}

fn generate_random_points_on_edges(
    points: &[(f64, f64)],
) -> (Vec<(f64, f64)>, Vec<(f64, f64)>) {
    let mut rng = fastrand::Rng::new();
    let n = points.len();
    
    let mut first_points = Vec::with_capacity(2);
    let mut second_points = Vec::with_capacity(2);
    
    for _ in 0..2 {

        let edge1_index = rng.usize(0..n);

        let mut edge2_index;
        loop {
            edge2_index = rng.usize(0..n);
            if edge2_index != edge1_index {
                break;
            }
        }
        
        let edge1_start = points[edge1_index];
        let edge1_end = points[(edge1_index + 1) % n];
        
        let edge2_start = points[edge2_index];
        let edge2_end = points[(edge2_index + 1) % n];
        
        let point1 = random_point_on_edge(edge1_start, edge1_end, &mut rng);
        let point2 = random_point_on_edge(edge2_start, edge2_end, &mut rng);
        
        first_points.push(point1);
        second_points.push(point2);
    }
    
    (first_points, second_points)
}

#[inline]
fn random_point_on_edge(
    start: (f64, f64),
    end: (f64, f64),
    rng: &mut fastrand::Rng
) -> (f64, f64) {
    let t: f64 = rng.f64();
    (
        start.0 + t * (end.0 - start.0),
        start.1 + t * (end.1 - start.1),
    )
}

fn is_intersect(
    seg1: &[(f64, f64)],
    seg2: &[(f64, f64)],
    polygon: &geo::Polygon,
) -> bool {
    let line1 = Line::new(
        Point::new(seg1[0].0, seg1[0].1),
        Point::new(seg2[0].0, seg2[0].1),
    );
    let line2 = Line::new(
        Point::new(seg1[1].0, seg1[1].1),
        Point::new(seg2[1].0, seg2[1].1),
    );

    match line_intersection(line1, line2) {
        Some(LineIntersection::SinglePoint { intersection: point, .. }) => {
            polygon.contains(&point)
        }
        _ => false,
    }
}

fn main() {
    let mut workbook = Workbook::new();
    
    let worksheet = workbook.add_worksheet();
    
    worksheet.write_string(0, 0, "n").expect("Failed to write header");
    worksheet.write_string(0, 1, "probability").expect("Failed to write header");
    worksheet.write_string(0, 2, "std_dev").expect("Failed to write header");
    worksheet.write_string(0, 3, "ci_lower").expect("Failed to write header");
    worksheet.write_string(0, 4, "ci_upper").expect("Failed to write header");
    worksheet.write_string(0, 5, "time_seconds").expect("Failed to write header");
    
    let iterations: i64 = 100_000_000;
    let mut row = 1;
    
    for n in 3..=2500 {
        println!("Processing n = {}", n);
        let start_time = Instant::now();
        
        let points = generate_polygon_points(n);
        let geo_points: Vec<_> = points.iter().map(|&(x, y)| Point::new(x, y)).collect();
        let polygon = Polygon::new(geo::LineString::from(geo_points), vec![]);

        let intersect_count = (0..iterations)
            .into_par_iter()
            .map(|_| {
                let (first_points, second_points) = generate_random_points_on_edges(&points);
                is_intersect(&first_points, &second_points, &polygon) as usize
            })
            .sum::<usize>();
        
        let probability = intersect_count as f64 / iterations as f64;
        let std_dev = (probability * (1.0 - probability) / iterations as f64).sqrt();
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
    }
    
    worksheet.autofit();
    
    workbook.save("polygon_intersection_results.xlsx").expect("Failed to save Excel file");
    
    println!("All polygon intersections calculated.");
}