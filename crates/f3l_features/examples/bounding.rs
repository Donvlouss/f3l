use f3l_core::glam::Vec3A;
#[cfg(feature = "app_kiss3d")]
use kiss3d::light::Light;
#[cfg(feature = "app_kiss3d")]
use kiss3d::window::Window;

use f3l_features::*;

#[cfg(not(feature = "app_kiss3d"))]
fn main() {
    println!("Add --features=app_kiss3d")
}

pub fn load_ply(path: &str) -> Vec<Vec3A> {
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

    vertices.into_iter().map(|v| Vec3A::new(v[0], v[1], v[2])).collect()
}


#[cfg(feature = "app_kiss3d")]
fn main() {
    use kiss3d::nalgebra::Point3;

    println!("Using Kiss3d app");

    let mut window = Window::new("Kiss3d: points");

    window.set_light(Light::StickToCamera);
    window.set_point_size(10.0); // (Not supported by all graphic drivers)

    let vertices = load_ply("../../data/table_voxel_down.ply");

    let obb = OBB::compute(&vertices);
    println!("OBB:\n{:?}",obb);
    let p0 = obb.center 
        - obb.primary * obb.length[0]
        - obb.secondary * obb.length[1]
        - obb.tertiary * obb.length[2];
    let p1 = p0 + obb.primary   * obb.length[0] * 2.;
    let p2 = p0 + obb.secondary * obb.length[1] * 2.;
    let p3 = p0 + obb.tertiary  * obb.length[2] * 2.;
    let p4 = p2 + obb.primary   * obb.length[0] * 2.;
    let p5 = p1 + obb.tertiary  * obb.length[2] * 2.;
    let p6 = p2 + obb.tertiary  * obb.length[2] * 2.;
    let p7 = p4 + obb.tertiary  * obb.length[2] * 2.;

    let aabb_bdx = aabb(&vertices);
    println!("Center: {}", (aabb_bdx.0 + aabb_bdx.1) / 2f32);
    let pp00 = (aabb_bdx.0 + aabb_bdx.1) / 2f32;
    let diff = aabb_bdx.1 - aabb_bdx.0;
    let l0 = diff.x.abs();
    let l1 = diff.y.abs();
    let l2 = diff.z.abs();
    let pp0 = pp00  
        - Vec3A::X * l0 / 2f32
        - Vec3A::Y * l1 / 2f32
        - Vec3A::Z * l2 / 2f32;
    let pp1 = pp0 + Vec3A::X * l0;
    let pp2 = pp0 + Vec3A::Y * l1;
    let pp3 = pp0 + Vec3A::Z * l2;
    let pp4 = pp2 + Vec3A::X * l0;
    let pp5 = pp1 + Vec3A::Z * l2;
    let pp6 = pp2 + Vec3A::Z * l2;
    let pp7 = pp4 + Vec3A::Z * l2;


    let o = Point3::<f32>::origin();
    let x = Point3::<f32>::new(1., 0., 0.);
    let y = Point3::<f32>::new(0., 1., 0.);
    let z = Point3::<f32>::new(0., 0., 1.);
    let xc = Point3::<f32>::new(1., 0., 0.);
    let yc = Point3::<f32>::new(0., 1., 0.);
    let zc = Point3::<f32>::new(0., 0., 1.);
    let line = Point3::<f32>::new(0., 1., 1.);
    let line1 = Point3::<f32>::new(1., 0., 1.);
    let pnt = Point3::<f32>::new(1., 1., 1.);

    let pts = [p0, p1, p2, p3, p4, p5, p6, p7];
    // pts.iter().for_each(|v| println!("{v}"));
    let pts1 = [pp0, pp1, pp2, pp3, pp4, pp5, pp6, pp7];
    let view_cloud = vertices.into_iter().map(|p| 
        Point3::new(p.x, p.y, p.z)).collect::<Vec<Point3<f32>>>();
    let view_pts = pts.iter()
        .map(|&p| Point3::new(p.x, p.y, p.z)).collect::<Vec<Point3<f32>>>();
    let view_pts1 = pts1.iter()
        .map(|&p| Point3::new(p.x, p.y, p.z)).collect::<Vec<Point3<f32>>>();
    let pairs = [
        (0_usize, 1_usize), (0, 2), (0, 3),
        (7, 4), (7, 5), (7, 6),
        (1, 4), (1, 5),
        (2, 4), (2, 6),
        (3, 5), (3, 6)
    ];

    while window.render() {
        window.draw_line(&o, &x, &xc);
        window.draw_line(&o, &y, &yc);
        window.draw_line(&o, &z, &zc);

        view_cloud.iter()
            .for_each(|p| {
                window.draw_point(p, &pnt)
            });

        pairs.iter()
            .for_each(|&(a, b)| {
                window.draw_line(
                    &view_pts[a], &view_pts[b], &line);
                window.draw_line(
                    &view_pts1[a], &view_pts1[b], &line1);
            });
    }

}