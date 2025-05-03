use rand::Rng;
use std::f64::consts::PI;
use geo::{Line, Polygon, Point, Contains};
use geo::line_intersection::{line_intersection, LineIntersection};

fn generate_polygon_points(n: usize) -> Vec<(f64, f64)> {
    let angle_step = 2.0 * PI / n as f64;
    let rotation_offset = if n % 2 == 0 { -PI / 2.0 } else { -PI / 2.0 - angle_step / 2.0 };
    
    (0..n).map(|i| {
        let angle = i as f64 * angle_step + rotation_offset;
        (angle.cos(), angle.sin())
    }).collect()
}

fn edges_are_geometrically_distinct(
    p1: (f64, f64), p2: (f64, f64),
    q1: (f64, f64), q2: (f64, f64)
) -> bool {
    let edge1_dx = p2.0 - p1.0;
    let edge1_dy = p2.1 - p1.1;
    
    let edge2_dx = q2.0 - q1.0;
    let edge2_dy = q2.1 - q1.1;
    
    let cross = edge1_dx * edge2_dy - edge1_dy * edge2_dx;
    
    cross.abs() > f64::EPSILON
}

fn generate_random_points_on_edges(
    points: &[(f64, f64)], 
    num_pairs: usize
) -> (Vec<(f64, f64)>, Vec<(f64, f64)>, Vec<(usize, usize)>) {
    let mut rng = rand::thread_rng();
    let n = points.len();
    let mut first_points = Vec::new();
    let mut second_points = Vec::new();
    let mut edge_indices = Vec::new();

    let valid_edges: Vec<usize> = (0..n)
        .filter(|&i| {
            let p1 = points[i];
            let p2 = points[(i + 1) % n];
            (p1.0 - p2.0).abs() > f64::EPSILON || (p1.1 - p2.1).abs() > f64::EPSILON
        })
        .collect();

    if valid_edges.len() < 2 {
        panic!("Polygon must have at least 2 valid edges");
    }

    for _ in 0..num_pairs {
        let idx1 = rng.gen_range(0..valid_edges.len());
        let edge1 = valid_edges[idx1];
        let (p1_start, p1_end) = (points[edge1], points[(edge1 + 1) % n]);
        
        let mut idx2 = rng.gen_range(0..valid_edges.len());
        loop {
            let candidate_edge = valid_edges[idx2];
            let (p2_start, p2_end) = (points[candidate_edge], points[(candidate_edge + 1) % n]);
            
            if edges_are_geometrically_distinct(p1_start, p1_end, p2_start, p2_end) {
                break;
            }
            idx2 = rng.gen_range(0..valid_edges.len());
        }
        let edge2 = valid_edges[idx2];
        let (p2_start, p2_end) = (points[edge2], points[(edge2 + 1) % n]);

        let t1: f64 = rng.gen();
        let x1 = p1_start.0 + t1 * (p1_end.0 - p1_start.0);
        let y1 = p1_start.1 + t1 * (p1_end.1 - p1_start.1);

        let t2: f64 = rng.gen();
        let x2 = p2_start.0 + t2 * (p2_end.0 - p2_start.0);
        let y2 = p2_start.1 + t2 * (p2_end.1 - p2_start.1);

        first_points.push((x1, y1));
        second_points.push((x2, y2));
        edge_indices.push((edge1, edge2));
    }

    (first_points, second_points, edge_indices)
}

fn is_intersect(
    seg1: &[(f64, f64)],
    seg2: &[(f64, f64)],
    polygon_coords: &[(f64, f64)],
) -> bool {
    let line1 = Line::new(
        Point::new(seg1[0].0, seg1[0].1),
        Point::new(seg2[0].0, seg2[0].1)
        ,
    );
    let line2 = Line::new(
        Point::new(seg1[1].0, seg1[1].1),
        Point::new(seg2[1].0, seg2[1].1),
    );

    let points: Vec<_> = polygon_coords.iter().map(|&(x, y)| Point::new(x, y)).collect();
    let polygon = Polygon::new(
        geo::LineString::from(points),
        vec![],
    );

    let intersection_result = line_intersection(line1, line2);

    match intersection_result {
        Some(LineIntersection::SinglePoint { intersection: point, .. }) => {
            polygon.contains(&point)
        }
        _ => {
            false
        },
    }
}

fn main() {
    let n = 4;
    let points = generate_polygon_points(n);
    let num_pairs = 2;
    let (first_points, second_points, edge_indices) = generate_random_points_on_edges(&points, num_pairs);

    println!("Polygon Points:");
    println!("polygon({:?})", points.iter().map(|(x,y)| format!("({:.6},{:.6})", x, y)).collect::<Vec<_>>().join(","));

    println!("\nPoint Pairs (Desmos format):");
    for i in 0..num_pairs {
        println!("Pair {}: polygon(({:.6},{:.6}),({:.6},{:.6})) on edges {:?}",
            i+1,
            first_points[i].0, first_points[i].1,
            second_points[i].0, second_points[i].1,
            edge_indices[i]
        );
    }

    println!("\nIntersection Check:");
    if is_intersect(&first_points, &second_points, &points) {
        println!("The segments intersect.");
    } else {
        println!("The segments do not intersect.");
    }
}