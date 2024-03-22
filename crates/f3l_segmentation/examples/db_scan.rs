#[cfg(feature = "app_kiss3d")]
use kiss3d::light::Light;
#[cfg(feature = "app_kiss3d")]
use kiss3d::window::Window;

use f3l_segmentation::*;

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

pub fn random_color() -> [f32; 3] {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    hsv_to_rgb(&[rng.gen_range(0f32..1f32), 1.0, 1.0])
}

#[cfg(feature = "app_kiss3d")]
fn main() {
    use kiss3d::nalgebra::Point3;

    println!("Using Kiss3d app");

    let mut window = Window::new("Kiss3d: points");

    window.set_light(Light::StickToCamera);
    window.set_point_size(10.0); // (Not supported by all graphic drivers)

    let vertices = load_ply("../../data/table_remove_plane.ply");

    let parameter = F3lClusterParameter {
        tolerance: 0.02f32,
        nb_in_tolerance: 20,
        min_nb_data: 100,
        max_nb_data: vertices.len(),
        max_nb_cluster: 5,
    };
    let mut extractor = DBScan::with_data(parameter, &vertices);
    let clusters = extractor.extract();

    let colors = (0..clusters.len())
        .map(|_| random_color())
        .collect::<Vec<_>>();
    let clusters = (0..clusters.len())
        .map(|i| extractor.at(i).unwrap())
        .collect::<Vec<_>>();

    println!("Nb of Clusters: {}", clusters.len());

    while window.render() {
        clusters.iter().zip(&colors).for_each(|(cluster, color)| {
            cluster.iter().for_each(|v| {
                window.draw_point(
                    &Point3::new(v[0], v[1], v[2]),
                    &Point3::new(color[0], color[1], color[2]),
                );
            })
        });
    }
}
