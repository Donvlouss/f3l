use f3l_core::glam::Vec3;
#[cfg(feature = "app_kiss3d")]
use kiss3d::light::Light;
#[cfg(feature = "app_kiss3d")]
use kiss3d::window::Window;

#[cfg(not(feature = "app_kiss3d"))]
fn main() {
    println!("Add --features=app_kiss3d")
}

pub fn load_ply(path: &str) -> Vec<Vec3> {
    use ply_rs as ply;
    use ply_rs::ply::Property;

    let mut f = std::fs::File::open(path).unwrap();
    // create a parser
    let p = ply::parser::Parser::<ply::ply::DefaultElement>::new();
    // use the parser: read the entire file
    let ply = p.read_ply(&mut f);
    // make sure it did work
    assert!(ply.is_ok());

    let ply_wrapper = ply.unwrap();

    let vertices = ply_wrapper.payload["vertex"]
        .iter()
        .map(|v| {
            let vertex = [v["x"].clone(), v["y"].clone(), v["z"].clone()];
            vertex
                .iter()
                .map(|v| match v {
                    Property::Float(f) => *f,
                    Property::Double(d) => *d as f32,
                    _ => 0f32,
                })
                .collect::<Vec<f32>>()
        })
        .collect::<Vec<Vec<f32>>>();

    vertices
        .into_iter()
        .map(|v| Vec3::new(v[0], v[1], v[2]))
        .collect()
}

#[allow(dead_code)]
pub fn hsv_to_rgb(hsv: &[f32; 3]) -> [f32; 3] {
    let [h, s, v] = *hsv;

    let hh = h * 6.;
    let hi = hh as usize;
    let f = hh - hi as f32;
    let p = v * (1. - s);
    let q = v * (1. - f * s);
    let t = v * (1. - (1. - f) * s);

    match hi {
        0 => [v, t, p],
        1 => [q, v, p],
        2 => [p, v, t],
        3 => [p, q, v],
        4 => [t, p, v],
        5 => [v, p, q],
        _ => [0., 0., 0.],
    }
}

#[allow(dead_code)]
pub fn random_color() -> [f32; 3] {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    hsv_to_rgb(&[rng.gen_range(0f32..1f32), 1.0, 1.0])
}

#[cfg(feature = "app_kiss3d")]
fn main() {
    use f3l_search_tree::*;
    use kiss3d::nalgebra::Point3;

    println!("Using Kiss3d app");

    let mut window = Window::new("Kiss3d: points");

    window.set_light(Light::StickToCamera);
    window.set_point_size(10.0); // (Not supported by all graphic drivers)

    let vertices = load_ply("../../data/table_voxel_down.ply");

    let mut tree = OcTree::with_data(&vertices, 100, 2);
    
    use std::time::Instant;
    let start = Instant::now();
    
    tree.build();

    let end = start.elapsed().as_micros();
    println!("Elapsed: {}", end);

    let mut colors: Vec<Point3::<f32>> = vec![];
    let mut points_cubes = vec![];
    let bdx = tree.nodes.iter().filter_map(|node| {
        let OcLeaf{ lower, upper, feature, points, .. }
            = node;
        match feature {
            OcFeature::Split(_) => {return None;},
            OcFeature::Leaf => {
                if points.is_empty() {
                    return None;
                }
            },
        };
        points_cubes.push(points.iter().map(|&i| Point3::new(vertices[i].x, vertices[i].y, vertices[i].z)
            ).collect::<Vec<Point3<f32>>>());
        colors.push(random_color().into());
        let p0 = *lower;
        let p1 = *upper;
        Some([
            Point3::new(p0[0], p0[1], p0[2]),
            Point3::new(p1[0], p0[1], p0[2]),
            Point3::new(p0[0], p1[1], p0[2]),
            Point3::new(p0[0], p0[1], p1[2]),

            Point3::new(p1[0], p1[1], p0[2]),
            Point3::new(p1[0], p0[1], p1[2]),
            Point3::new(p0[0], p1[1], p1[2]),
            Point3::new(p1[0], p1[1], p1[2]),
        ])
    }).collect::<Vec<_>>();
    let pairs = [
        (0_usize, 1_usize),
        (0, 2),
        (0, 3),
        (7, 4),
        (7, 5),
        (7, 6),
        (1, 4),
        (1, 5),
        (2, 4),
        (2, 6),
        (3, 5),
        (3, 6),
    ];

    let o = Point3::<f32>::origin();
    let x = Point3::<f32>::new(1., 0., 0.);
    let y = Point3::<f32>::new(0., 1., 0.);
    let z = Point3::<f32>::new(0., 0., 1.);
    let xc = Point3::<f32>::new(1., 0., 0.);
    let yc = Point3::<f32>::new(0., 1., 0.);
    let zc = Point3::<f32>::new(0., 0., 1.);

    while window.render() {
        window.draw_line(&o, &x, &xc);
        window.draw_line(&o, &y, &yc);
        window.draw_line(&o, &z, &zc);

        (0..bdx.len()).for_each(|i| {
            let color = &colors[i];
            points_cubes[i].iter().for_each(|p| {
                window.draw_point(p, color);
            });
            pairs.iter().for_each(|&(a, b)| {
                window.draw_line(
                    &bdx[i][a],
                    &bdx[i][b],
                    color
                );
            });
        });
    }

}