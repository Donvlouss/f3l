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

fn load_img(path: &str) -> Vec<[f32; 2]> {
    use image::GenericImageView;
    let img = image::open(path).unwrap();
    let dimension = img.dimensions();
    img.pixels().into_iter().enumerate().filter_map(|(i, (x, y, rgb))| {
        if i % 10  != 0 {
            return None;
        }
        let a = rgb.0;
        if a.iter().all(|&c| c != 0) {
            Some([
                x as f32 / dimension.0 as f32,
                y as f32 / dimension.1 as f32,
            ])
        } else {
            None
        }
    }).collect::<Vec<_>>()
}

#[cfg(feature = "app_kiss3d")]
fn main() {
    use f3l_surface::Delaunay2D;

    println!("Using Kiss3d app");

    let mut window = Window::new("Kiss3d: points");

    window.set_light(Light::StickToCamera);
    window.set_point_size(10.0); // (Not supported by all graphic drivers)

    // let points = load_img("../../data/hull.png");
    let points = load_img("../../data/hull_hole.png");

    let mut solver = Delaunay2D::new(&points);

    solver.compute(0.005);
    // solver.compute(6.);

    let view_points = points.iter().map(|&p| {
        Point3::new(p[0], p[1], 0.)
    }).collect::<Vec<_>>();

    let o = Point3::<f32>::origin();
    let x = Point3::<f32>::new(1., 0., 0.);
    let y = Point3::<f32>::new(0., 1., 0.);
    let z = Point3::<f32>::new(0., 0., 1.);
    let xc = Point3::<f32>::new(1., 0., 0.);
    let yc = Point3::<f32>::new(0., 1., 0.);
    let zc = Point3::<f32>::new(0., 0., 1.);
    let w = Point3::<f32>::new(1., 1., 1.);

    let activate = solver.triangles.iter().flat_map(|tri| {
        [(0, 1), (0, 2), (1, 2)].iter().map(|&(e0, e1)| {
            (
                Point3::new(points[tri.point[e0]][0], points[tri.point[e0]][1], 0.),
                Point3::new(points[tri.point[e1]][0], points[tri.point[e1]][1], 0.),
            )
        }).collect::<Vec<_>>()
    }).collect::<Vec<_>>();

    let mut contour_color: Vec<Point3<f32>> = vec![];
    let alpha_edge = solver.contours.iter().map(|wire| {
        contour_color.push(random_color().into());
        wire.iter().map(|&(e0, e1)| {
            (
                &view_points[e0],
                &view_points[e1],
            )
        }).collect::<Vec<_>>()
    }).collect::<Vec<_>>();

    while window.render() {
        window.draw_line(&o, &x, &xc);
        window.draw_line(&o, &y, &yc);
        window.draw_line(&o, &z, &zc);


        view_points.iter().for_each(|p| {
                window.draw_point(&p, &yc);
        });
        
        activate.iter().for_each(|e| {
            window.draw_line(&e.0, &e.1, &w);
        });

        alpha_edge.iter().zip(&contour_color).for_each(|(wire, c)| {
            wire.iter().for_each(|&(e0, e1)| {
                window.draw_line(e0, e1, c);
            })
        });
    }

}
