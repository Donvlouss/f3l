#[cfg(feature = "app_kiss3d")]
use kiss3d::light::Light;
#[cfg(feature = "app_kiss3d")]
use kiss3d::window::Window;

use f3l_segmentation::{
    sac_algorithm::*,
    sac_model::*,
};

#[cfg(not(feature = "app_kiss3d"))]
fn main() {
    println!("Add --features=app_kiss3d")
}

mod util;
use util::load_ply;

#[cfg(feature = "app_kiss3d")]
fn main() {
    use kiss3d::nalgebra::Point3;
    println!("Using Kiss3d app");

    let mut window = Window::new("Kiss3d: points");

    window.set_light(Light::StickToCamera);
    window.set_point_size(10.0); // (Not supported by all graphic drivers)

    let vertices = load_ply("data/table_voxel_down.ply");

    let parameter = SacAlgorithmParameter {
        probability: 0.99, 
        threshold: 0.02, 
        max_iterations: 2000,
        threads: 1,
    };
    let mut model = SacModelPlane::with_data(&vertices);
    let mut algorithm = SacRansac{
        parameter,
        inliers: vec![],
    };

    use std::time::Instant;
    let start = Instant::now();
    
    let result = algorithm.compute(&mut model);

    let end = start.elapsed().as_millis();
    println!("Elapsed: {}", end);

    if !result {
        println!("Segmentation Failed");
        return;
    }

    let factor = model.get_coefficient();
    let inlier = algorithm.inliers;
    println!("Plane Coefficients: {:?}", factor);
    println!("Nb Inliers : {}", inlier.len());

    let inlier = inlier
        .iter()
        .map(|id| {
            let p = vertices[*id];
            Point3::<f32>::new(p[0], p[1], p[2])
        }).collect::<Vec<_>>();
    let total = vertices
        .iter()
        .map(|p| {
            Point3::<f32>::new(p[0], p[1], p[2])
        }).collect::<Vec<_>>();

    let green = Point3::<f32>::new(0., 1., 0.);
    let white = Point3::<f32>::new(1., 1., 1.);

    while window.render() {
        total.iter()
            .for_each(|p| {
                window.draw_point(p, &white);
            });
        inlier.iter()
            .for_each(|p| {
                window.draw_point(p, &green);
            });
    }
}