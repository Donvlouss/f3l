#[cfg(feature = "app_kiss3d")]
use kiss3d::light::Light;
#[cfg(feature = "app_kiss3d")]
use kiss3d::nalgebra::Point3;
#[cfg(feature = "app_kiss3d")]
use kiss3d::window::Window;

use f3l_filter::*;

#[cfg(not(feature = "app_kiss3d"))]
fn main() {
    println!("Add --features=app_kiss3d")
}

pub fn load_ply(path: &str) -> Vec<[f32; 3]> {
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

    vertices.into_iter().map(|v| [v[0], v[1], v[2]]).collect()
}

#[cfg(feature = "app_kiss3d")]
fn main() {
    use std::ops::Bound;

    println!("Using Kiss3d app");

    let mut window = Window::new("Kiss3d: points");

    window.set_light(Light::StickToCamera);
    window.set_point_size(10.0); // (Not supported by all graphic drivers)

    let vertices = load_ply("../../data/table_scene_lms400.ply");

    let range = vec![
        (0, Bound::Included(-0.3)..Bound::Included(0.5)),
        (1, Bound::Included(0.)..Bound::Included(0.8)),
        (2, Bound::Included(-1.4)..Bound::Included(-1.3)),
    ];
    let mut filter = ConditionRemoval::with_data(&range);

    let out = filter.filter_instance(&vertices);

    while window.render() {
        out.iter().for_each(|v| {
            window.draw_point(&Point3::new(v[0], v[1], v[2]), &Point3::new(0.0, 1.0, 0.0));
        });
    }
}
