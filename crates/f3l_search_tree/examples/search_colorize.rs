#[cfg(all(feature="core", not(feature="pure")))]
use f3l_core::glam::Vec3;
#[cfg(feature = "app_kiss3d")]
use kiss3d::light::Light;
#[cfg(feature = "app_kiss3d")]
use kiss3d::window::Window;

#[cfg(not(feature = "app_kiss3d"))]
fn main() {
    println!("Add --features=app_kiss3d")
}

#[cfg(all(feature="core", not(feature="pure")))]
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
    use f3l_core::{compute_covariance_matrix, jacobi_eigen_square_n};
    use f3l_search_tree::*;
    use kiss3d::nalgebra::Point3;
    use rand::Rng;

    println!("Using Kiss3d app");

    let mut window = Window::new("Kiss3d: points");

    window.set_light(Light::StickToCamera);
    window.set_point_size(10.0); // (Not supported by all graphic drivers)

    let vertices = load_ply("../../data/table_voxel_down.ply");

    let normal_len = 0.04f32;
    let mut tree = KdTree::with_data(&vertices);
    tree.build();

    let search_range = 0.08f32;
    let nb_sample = 10_usize;
    let mut rng = rand::thread_rng();

    let mut seeds = std::collections::BTreeSet::<usize>::new();
    while seeds.len() < nb_sample {
        let id = rng.gen_range(0..vertices.len());
        if seeds.contains(&id) {
            continue;
        }
        seeds.insert(id);
    }

    let mut cloud_mask = vec![true; vertices.len()];

    let by = if search_range == 0.0 {
        SearchBy::Count(1)
    } else {
        SearchBy::Radius(search_range * search_range)
    };
    let mut result = TreeRadiusResult::new(search_range * search_range);
    let samples = seeds
        .iter()
        .map(|&seed| {
            result.clear();
            tree.search(vertices[seed], by, &mut result);
            result
                .data
                .iter()
                .map(|&i| {
                    cloud_mask[i] = false;
                    Point3::new(vertices[i].x, vertices[i].y, vertices[i].z)
                })
                .collect::<Vec<Point3<f32>>>()
        })
        .collect::<Vec<_>>();
    let normals = samples
        .iter()
        .zip(&seeds)
        .map(|(cluster, &seed)| {
            let cov = compute_covariance_matrix(cluster);
            // let mat = cov.to_rows_array_2d();
            let eigen = jacobi_eigen_square_n(cov.0);
            (
                Point3::new(vertices[seed].x, vertices[seed].y, vertices[seed].z),
                Point3::new(
                    vertices[seed].x + normal_len * eigen[0].eigenvector[0],
                    vertices[seed].y + normal_len * eigen[0].eigenvector[1],
                    vertices[seed].z + normal_len * eigen[0].eigenvector[2],
                ),
                Point3::new(
                    vertices[seed].x + normal_len * eigen[1].eigenvector[0],
                    vertices[seed].y + normal_len * eigen[1].eigenvector[1],
                    vertices[seed].z + normal_len * eigen[1].eigenvector[2],
                ),
                Point3::new(
                    vertices[seed].x + normal_len * eigen[2].eigenvector[0],
                    vertices[seed].y + normal_len * eigen[2].eigenvector[1],
                    vertices[seed].z + normal_len * eigen[2].eigenvector[2],
                ),
            )
        })
        .collect::<Vec<(Point3<f32>, Point3<f32>, Point3<f32>, Point3<f32>)>>();
    let colors = (0..nb_sample)
        .map(|_| random_color().into())
        .collect::<Vec<Point3<f32>>>();

    let cloud = vertices
        .into_iter()
        .zip(cloud_mask)
        .filter(|&(_, m)| m)
        .map(|(v, _)| Point3::new(v.x, v.y, v.z))
        .collect::<Vec<Point3<f32>>>();

    let o = Point3::<f32>::origin();
    let x = Point3::<f32>::new(1., 0., 0.);
    let y = Point3::<f32>::new(0., 1., 0.);
    let z = Point3::<f32>::new(0., 0., 1.);
    let xc = Point3::<f32>::new(1., 0., 0.);
    let yc = Point3::<f32>::new(0., 1., 0.);
    let zc = Point3::<f32>::new(0., 0., 1.);
    let w = Point3::<f32>::new(1., 1., 1.);

    while window.render() {
        window.draw_line(&o, &x, &xc);
        window.draw_line(&o, &y, &yc);
        window.draw_line(&o, &z, &zc);

        cloud.iter().for_each(|v| window.draw_point(v, &w));
        samples.iter().zip(&colors).for_each(|(cluster, c)| {
            cluster.iter().for_each(|v| window.draw_point(v, c));
        });
        normals.iter().for_each(|(a, b, c, d)| {
            window.draw_line(a, b, &x);
            window.draw_line(a, c, &y);
            window.draw_line(a, d, &z);
        });
    }
}
