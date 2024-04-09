#[cfg(feature = "app_kiss3d")]
use kiss3d::light::Light;
#[cfg(feature = "app_kiss3d")]
use kiss3d::window::Window;
#[cfg(feature = "app_kiss3d")]
use kiss3d::nalgebra::Point3;

#[cfg(not(feature = "app_kiss3d"))]
fn main() {
    println!("Add --features=app_kiss3d")
}

pub fn load_ply(path: &str) -> Vec<Point3<f32>> {
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

    vertices.into_iter().map(|v| [v[0], v[1], v[2]].into()).collect()
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
    use f3l_surface::*;

    println!("Using Kiss3d app");

    let mut window = Window::new("Kiss3d: points");

    window.set_light(Light::StickToCamera);
    window.set_point_size(10.0); // (Not supported by all graphic drivers)

    let vertices = load_ply("../../data/table_voxel_down.ply");

    // Use Wrapper to handle 2d or 3d
    let mut cvh = ConvexHull::new(&vertices);

    // Or use `ConvexHull3D` directly.
    // let mut cvh = ConvexHull3D::new(&vertices);

    cvh.compute();
    let hulls = if let ConvexHullId::D3(hulls) = cvh.hulls() {
        hulls
    } else {
        panic!("Could resolve to D3 type.")
    };

    let triangles = hulls.iter().map(|tri| {
        let &FaceIdType { point: [p0, p1, p2] } = tri;
        [
            Point3::new(vertices[p0][0], vertices[p0][1], vertices[p0][2]),
            Point3::new(vertices[p1][0], vertices[p1][1], vertices[p1][2]),
            Point3::new(vertices[p2][0], vertices[p2][1], vertices[p2][2]),
        ]
    }).collect::<Vec<_>>();
    let colors = (0..triangles.len()).map(|_| random_color().into()).collect::<Vec<Point3<f32>>>();

    let o = Point3::<f32>::origin();
    let x = Point3::<f32>::new(5., 0., 0.);
    let y = Point3::<f32>::new(0., 5., 0.);
    let z = Point3::<f32>::new(0., 0., 5.);
    let xc = Point3::<f32>::new(1., 0., 0.);
    let yc = Point3::<f32>::new(0., 1., 0.);
    let zc = Point3::<f32>::new(0., 0., 1.);
    let w = Point3::<f32>::new(1., 1., 1.);

    while window.render() {
        window.draw_line(&o, &x, &xc);
        window.draw_line(&o, &y, &yc);
        window.draw_line(&o, &z, &zc);

        vertices.iter().for_each(|p| {
            window.draw_point(&p, &w);
        });

        triangles.iter().zip(&colors).for_each(|(t, c)| {
            window.draw_line(&t[0], &t[1], c);
            window.draw_line(&t[0], &t[2], c);
            window.draw_line(&t[1], &t[2], c);
        });
    }
}