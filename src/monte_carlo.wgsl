struct SimulationParams {
    n: u32,
    iterations_per_thread: u32,
    seed: u32,
    _padding: u32,
}

struct Point2D {
    x: f32,
    y: f32,
}

@group(0) @binding(0) var<uniform> params: SimulationParams;
@group(0) @binding(1) var<storage, read> polygon: array<Point2D>;
@group(0) @binding(2) var<storage, read_write> results: array<u32>;

fn pcg(state: ptr<function, u32>) -> f32 {
    let oldstate: u32 = *state;
    *state = oldstate * 747796405u + 2891336453u;
    let word: u32 = ((oldstate >> ((oldstate >> 28u) + 4u)) ^ oldstate) * 277803737u;
    return f32((word >> 22u) ^ word) / 4294967295.0;
}

fn random_point_on_edge(edge_idx: u32, state: ptr<function, u32>) -> Point2D {
    let start = polygon[edge_idx];
    let end = polygon[(edge_idx + 1u) % params.n];
    let t = pcg(state);
    
    var result: Point2D;
    result.x = start.x + t * (end.x - start.x);
    result.y = start.y + t * (end.y - start.y);
    return result;
}

fn is_point_in_polygon(p: Point2D) -> bool {
    var inside = false;
    for (var i = 0u; i < params.n; i = i + 1u) {
        let j = (i + 1u) % params.n;
        let vi = polygon[i];
        let vj = polygon[j];
        
        if (((vi.y > p.y) != (vj.y > p.y)) && 
            (p.x < (vj.x - vi.x) * (p.y - vi.y) / (vj.y - vi.y) + vi.x)) {
            inside = !inside;
        }
    }
    return inside;
}

fn line_segments_intersect(p1: Point2D, p2: Point2D, p3: Point2D, p4: Point2D) -> Point2D {
    let s1_x = p2.x - p1.x;
    let s1_y = p2.y - p1.y;
    let s2_x = p4.x - p3.x;
    let s2_y = p4.y - p3.y;
    
    let div = (-s2_x * s1_y + s1_x * s2_y);
    if (abs(div) < 0.0001) {
        var no_intersection: Point2D;
        no_intersection.x = -1000.0;
        no_intersection.y = -1000.0;
        return no_intersection;
    }
    
    let s = (-s1_y * (p1.x - p3.x) + s1_x * (p1.y - p3.y)) / div;
    let t = (s2_x * (p1.y - p3.y) - s2_y * (p1.x - p3.x)) / div;
    
    if (s >= 0.0 && s <= 1.0 && t >= 0.0 && t <= 1.0) {
        var intersection: Point2D;
        intersection.x = p1.x + (t * s1_x);
        intersection.y = p1.y + (t * s1_y);
        return intersection;
    }
    
    var no_intersection: Point2D;
    no_intersection.x = -1000.0;
    no_intersection.y = -1000.0;
    return no_intersection;
}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    var rng_state: u32 = params.seed + idx * 1664525u + 1013904223u;
    
    var intersect_count: u32 = 0u;
    
    let batch_size = 16u;
    let full_batches = params.iterations_per_thread / batch_size;
    
    for (var batch = 0u; batch < full_batches; batch = batch + 1u) {
        for (var j = 0u; j < batch_size; j = j + 1u) {
            let edge1_idx = u32(pcg(&rng_state) * f32(params.n));
            var edge2_idx = u32(pcg(&rng_state) * f32(params.n));
            while (edge2_idx == edge1_idx) {
                edge2_idx = u32(pcg(&rng_state) * f32(params.n));
            }
            
            let p1 = random_point_on_edge(edge1_idx, &rng_state);
            let p2 = random_point_on_edge(edge2_idx, &rng_state);
            
            let edge3_idx = u32(pcg(&rng_state) * f32(params.n));
            var edge4_idx = u32(pcg(&rng_state) * f32(params.n));
            while (edge4_idx == edge3_idx) {
                edge4_idx = u32(pcg(&rng_state) * f32(params.n));
            }
            
            let p3 = random_point_on_edge(edge3_idx, &rng_state);
            let p4 = random_point_on_edge(edge4_idx, &rng_state);
            
            let intersection = line_segments_intersect(p1, p2, p3, p4);
            if (intersection.x > -999.0 && is_point_in_polygon(intersection)) {
                intersect_count = intersect_count + 1u;
            }
        }
    }
    
    for (var i = full_batches * batch_size; i < params.iterations_per_thread; i = i + 1u) {
        let edge1_idx = u32(pcg(&rng_state) * f32(params.n));
        var edge2_idx = u32(pcg(&rng_state) * f32(params.n));
        while (edge2_idx == edge1_idx) {
            edge2_idx = u32(pcg(&rng_state) * f32(params.n));
        }
        
        let p1 = random_point_on_edge(edge1_idx, &rng_state);
        let p2 = random_point_on_edge(edge2_idx, &rng_state);
        
        let edge3_idx = u32(pcg(&rng_state) * f32(params.n));
        var edge4_idx = u32(pcg(&rng_state) * f32(params.n));
        while (edge4_idx == edge3_idx) {
            edge4_idx = u32(pcg(&rng_state) * f32(params.n));
        }
        
        let p3 = random_point_on_edge(edge3_idx, &rng_state);
        let p4 = random_point_on_edge(edge4_idx, &rng_state);
        
        let intersection = line_segments_intersect(p1, p2, p3, p4);
        if (intersection.x > -999.0 && is_point_in_polygon(intersection)) {
            intersect_count = intersect_count + 1u;
        }
    }
    
    results[idx] = intersect_count;
}