#[cfg(feature = "app_kiss3d")]
use kiss3d::light::Light;
#[cfg(feature = "app_kiss3d")]
use kiss3d::nalgebra::Point3;
#[cfg(feature = "app_kiss3d")]
use kiss3d::window::Window;

#[cfg(not(feature = "app_kiss3d"))]
fn main() {
    println!("Add --features=app_kiss3d")
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
fn load_img(path: &str) -> Vec<[f32; 2]> {
    use image::GenericImageView;
    let img = image::open(path).unwrap();
    let dimension = img.dimensions();
    img.pixels()
        .into_iter()
        .enumerate()
        .filter_map(|(i, (x, y, rgb))| {
            if i % 10 != 0 {
                return None;
            }
            let a = rgb.0;
            if a.iter().all(|&c| c != 0) {
                Some([x as f32 / dimension.0 as f32, y as f32 / dimension.1 as f32])
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}

#[cfg(feature = "app_kiss3d")]
fn main() {
    use f3l_surface::{Delaunay2D, Delaunay2DShape, FaceIdType};

    println!("Using Kiss3d app");
    println!("Vertex Voxel Down 10x, so edges may not be straight.");

    let mut window = Window::new("Kiss3d: points");

    window.set_light(Light::StickToCamera);
    window.set_point_size(10.0); // (Not supported by all graphic drivers)

    // Four pictures could be used to show example.
    // let points = load_img("../../data/hull.png");
    // let points = load_img("../../data/hull_hole.png");
    // let points = load_img("../../data/hull_hole_multiple.png");
    let points = load_img("../../data/hull_hole_multiples.png");

    let mut solver = Delaunay2D::new(&points);

    solver.compute(0.005);

    let view_points = points
        .iter()
        .map(|&p| Point3::new(p[0], p[1], 0.))
        .collect::<Vec<_>>();

    let o = Point3::<f32>::origin();
    let x = Point3::<f32>::new(1., 0., 0.);
    let y = Point3::<f32>::new(0., 1., 0.);
    let z = Point3::<f32>::new(0., 0., 1.);
    let xc = Point3::<f32>::new(1., 0., 0.);
    let yc = Point3::<f32>::new(0., 1., 0.);
    let zc = Point3::<f32>::new(0., 0., 1.);
    let w = Point3::<f32>::new(1., 1., 1.);

    let shapes = solver.shapes;

    let shapes = shapes
        .into_iter()
        .map(|shape| {
            let Delaunay2DShape { mesh, contours } = shape;
            let c: Point3<f32> = random_color().into();

            let mesh = mesh
                .into_iter()
                .flat_map(|FaceIdType { point }| {
                    [(0, 1), (0, 2), (1, 2)]
                        .into_iter()
                        .map(|(a, b)| (view_points[point[a]], view_points[point[b]]))
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            let contours = contours
                .into_iter()
                .flat_map(|contour| {
                    contour
                        .into_iter()
                        .map(|(a, b)| (view_points[a], view_points[b]))
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            (c, mesh, contours)
        })
        .collect::<Vec<_>>();

    while window.render() {
        window.draw_line(&o, &x, &xc);
        window.draw_line(&o, &y, &yc);
        window.draw_line(&o, &z, &zc);

        shapes.iter().for_each(|(color, mesh, contours)| {
            mesh.iter().for_each(|(a, b)| {
                window.draw_line(a, b, color);
            });
            contours.iter().for_each(|(a, b)| {
                window.draw_line(a, b, &w);
            });
        });
    }
}
