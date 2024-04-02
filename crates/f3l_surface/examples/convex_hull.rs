#[cfg(feature = "app_kiss3d")]
use kiss3d::light::Light;
#[cfg(feature = "app_kiss3d")]
use kiss3d::window::Window;

#[cfg(not(feature = "app_kiss3d"))]
fn main() {
    println!("Add --features=app_kiss3d")
}

#[cfg(feature = "app_kiss3d")]
fn main() {
    use kiss3d::nalgebra::Point3;
    use f3l_surface::*;
    use image::GenericImageView;

    println!("Using Kiss3d app");

    let mut window = Window::new("Kiss3d: points");

    window.set_light(Light::StickToCamera);
    window.set_point_size(10.0); // (Not supported by all graphic drivers)

    let img = image::open("../../data/hull.png").unwrap();
    let dimension = img.dimensions();
    let points = img.pixels().into_iter().filter_map(|(x, y, rgb)| {
        let a = rgb.0;
        if a.iter().all(|&c| c != 0) {
            Some([
                x as f32 / dimension.0 as f32,
                y as f32 / dimension.1 as f32,
            ])
        } else {
            None
        }
    }).collect::<Vec<_>>();

    let mut cvh = ConvexHull2D::new(&points);
    cvh.compute();
    let hulls = cvh.hulls;

    let view_points = points.iter().map(|&p| {
        Point3::new(p[0], p[1], 0.)
    }).collect::<Vec<_>>();
    let lines = hulls.iter().map(|&i| {
        view_points[i]
    }).collect::<Vec<_>>();

    let o = Point3::<f32>::origin();
    let x = Point3::<f32>::new(1., 0., 0.);
    let y = Point3::<f32>::new(0., 1., 0.);
    let z = Point3::<f32>::new(0., 0., 1.);
    let xc = Point3::<f32>::new(1., 0., 0.);
    let yc = Point3::<f32>::new(0., 1., 0.);
    let zc = Point3::<f32>::new(0., 0., 1.);
    let w = Point3::<f32>::new(1., 1., 1.);
    let line = Point3::<f32>::new(0., 1., 1.);

    while window.render() {
        window.draw_line(&o, &x, &xc);
        window.draw_line(&o, &y, &yc);
        window.draw_line(&o, &z, &zc);


        view_points.iter().for_each(|p| {
                window.draw_point(&p, &w);
        });

        (0..lines.len()).for_each(|i| {
            if i == lines.len()-1 {
                window.draw_line(&lines[i], &lines[0], &line);
            } else {
                window.draw_line(&lines[i], &lines[i+1], &line);
            }
        });
    }

}