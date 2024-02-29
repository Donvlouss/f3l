#[cfg(feature = "app_kiss3d")]
use kiss3d::light::Light;
#[cfg(feature = "app_kiss3d")]
use kiss3d::window::Window;
#[cfg(feature = "app_kiss3d")]
use nalgebra::Point3;

mod util;
use util::load_ply;

use f3l_filter::*;

#[cfg(not(feature = "app_kiss3d"))]
fn main() {
    println!("Add --features=app_kiss3d")
}

#[cfg(feature = "app_kiss3d")]
fn main() {
    println!("Using Kiss3d app");

    let mut window = Window::new("Kiss3d: points");

    window.set_light(Light::StickToCamera);
    window.set_point_size(10.0); // (Not supported by all graphic drivers)

    let vertices = load_ply("data/Itable_scene_lms400.ply");

    let mut filter = PassThrough::with_data(&vertices, (Bound::Included(0.), Bound::Included(0.5)), 0);
    use std::{ops::Bound, time::Instant};
    let start = Instant::now();

    let out = filter.filter_instance();

    let end = start.elapsed().as_millis();
    println!("Elapsed: {}", end);

    while window.render() {
        out.iter()
            .for_each(|v| {
                window.draw_point(&Point3::new(v[0], v[1], v[2]), &Point3::new(0.0, 1.0, 0.0));
            });
    }
}