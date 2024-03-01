#[cfg(feature = "app_kiss3d")]
use kiss3d::light::Light;
#[cfg(feature = "app_kiss3d")]
use kiss3d::window::Window;

mod util;
use util::load_ply;

use f3l_segmentation::*;

#[cfg(not(feature = "app_kiss3d"))]
fn main() {
    println!("Add --features=app_kiss3d")
}

#[cfg(feature = "app_kiss3d")]
fn main() {
    use kiss3d::nalgebra::Point3;
    use util::random_color;

    println!("Using Kiss3d app");

    let mut window = Window::new("Kiss3d: points");

    window.set_light(Light::StickToCamera);
    window.set_point_size(10.0); // (Not supported by all graphic drivers)

    let vertices = load_ply("data/table_remove_plane.ply");

    let parameter = F3lClusterParameter {
        tolerance: 0.02f32,
        nb_in_tolerance: 1,
        min_nb_data: 100,
        max_nb_data: 25000,
        max_nb_cluster: 5,
    };
    let mut extractor = EuclideanClusterExtractor::with_data(parameter, &vertices);
    let clusters = extractor.extract();
    let colors = (0..clusters.len())
        .map(|_| random_color())
        .collect::<Vec<_>>();
    let clusters = (0..clusters.len())
        .map(|i| extractor.at(i).unwrap())
        .collect::<Vec<_>>();

    println!("Nb of Clusters: {}", clusters.len());

    while window.render() {
        clusters.iter().zip(&colors)
            .for_each(|(cluster, color)| {
                cluster.iter()
                    .for_each(|v| {
                        window.draw_point(
                            &Point3::new(v[0], v[1], v[2]),
                            &Point3::new(color[0], color[1], color[2])
                        );
                    })

            });
    }
}