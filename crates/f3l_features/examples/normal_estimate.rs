use f3l_core::glam::Vec3;
#[cfg(feature = "app_kiss3d")]
use kiss3d::light::Light;
#[cfg(feature = "app_kiss3d")]
use kiss3d::window::Window;

use f3l_features::*;

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

#[cfg(feature = "app_kiss3d")]
fn main() {
    use f3l_search_tree::SearchBy;
    use kiss3d::nalgebra::{Point3, Vector3};

    println!("Using Kiss3d app");

    let mut window = Window::new("Kiss3d: points");

    window.set_light(Light::StickToCamera);
    window.set_point_size(10.0); // (Not supported by all graphic drivers)

    let vertices = load_ply("../../data/table_voxel_down.ply");
    let normal_len = 0.02f32;

    // Use radius or knn to search neighbors
    // let mut estimator = NormalEstimation::with_data(SearchBy::Radius(0.08f32), &vertices);
    let mut estimator = NormalEstimation::new(SearchBy::Count(10));

    if !estimator.compute(&vertices) {
        println!("Compute Normal Failed. Exit...");
        return;
    }
    let normals = estimator.normals();

    let color_v = Point3::new(1.0, 1.0, 1.0);
    let color_n = Point3::new(0.0, 1.0, 1.0);

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

        vertices
            .iter()
            .zip(&normals)
            .enumerate()
            .for_each(|(_, (v, n))| {
                let p = Point3::new(v[0], v[1], v[2]);
                window.draw_point(&p, &color_v);
                if let Some(n) = n {
                    let dir = if n[1] < 0f32 { -1f32 } else { 1. };
                    let p1 = p + Vector3::new(n[0], n[1], n[2]) * normal_len * dir;
                    window.draw_line(&p, &p1, &color_n);
                };
            });
    }
}
