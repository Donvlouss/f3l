use f3l_segmentation::sac_model::*;
use f3l_segmentation::sac_algorithm::*;

#[cfg(test)]
mod sac_segmentation {
    use super::*;

    mod plane {
        use super::*;
        use ply_rs as ply;
        use ply_rs::ply::Property;

        fn parse_out(p: Property) -> f32 {
            match p {
                Property::Float(f) => f,
                Property::Double(d) => d as f32,
                _ => 0.
            }
        }

        fn load_ply(path: &str) -> Vec<[f32; 3]> {
            let f = std::fs::File::open(path).unwrap();
            let mut f = std::io::BufReader::new(f);
            // create a parser
            let p = ply::parser::Parser::<ply::ply::DefaultElement>::new();
            // use the parser: read the entire file
            let ply = p.read_ply(&mut f);
            // make sure it did work
            assert!(ply.is_ok());

            let ply_wrapper = ply.unwrap();

            let vs = ply_wrapper.payload["vertex"].iter()
                .map(|v| {
                    [
                        parse_out(v["x"].clone()),
                        parse_out(v["y"].clone()),
                        parse_out(v["z"].clone()),
                    ]
                })
                .collect::<Vec<_>>();
            vs
        }

        #[test]
        fn pnt_to_plane() {
            let p = [0f32, 0f32, 1.3];
            let distance = SacModelPlane::compute_point_to_model(p, &[0f32, 0., 1., 0.]);
            assert!( ((p[2]-distance)*1000.).round() < f32::EPSILON );
        }

        #[test]
        fn sac_plane() {
            use std::path::Path;
            if !Path::new("data/table_voxel_down.ply").exists() {
                return;
            }
            let vertices = load_ply("data/table_voxel_down.ply");
            let parameter = SacAlgorithmParameter {
                probability: 0.99, 
                threshold: 0.02, 
                max_iterations: 200,
                threads: 1
            };
            let mut model = SacModelPlane::with_data(&vertices);
            let mut algorithm = SacRansac{
                parameter,
                inliers: vec![],
            };
            let result = algorithm.compute(&mut model);
            let _factor = model.get_coefficient();
            assert!(result);
        }
    }

    mod line {
        use super::*;

        #[test]
        fn sac_line_1() {
            let line = (0..10).map(|i| [i as f32; 3]).collect::<Vec<_>>();
            let line = [line, vec![
                [1.5, 1., 1.],
                [2., 2.5, 2.],
                [3., 3., 3.5]
            ]].concat();
            let parameter = SacAlgorithmParameter {
                probability: 0.99, 
                threshold: 0.1, 
                max_iterations: 200,
                threads: 1
            };
            let mut model = SacModelLine::with_data(&line);
            let mut algorithm = SacRansac{
                parameter,
                inliers: vec![],
            };
            let result = algorithm.compute(&mut model);
            let _factor = model.get_coefficient();
            let inliers = &algorithm.inliers;
            assert!(result);
            assert_eq!(inliers.len(), 10);
        }

        #[test]
        fn sac_line_2() {
            use rand::Rng;
            let mut rng = rand::thread_rng();

            let nb_total = 1000usize;
            let nb_init_inliers = 600usize;
            let mut nb_inliers = nb_init_inliers;
            let line = (0..nb_total - nb_init_inliers)
                .map(|i| {
                    let y = rng.gen_range(-1f32..=1.);
                    if y.abs() <= 0.1 {
                        nb_inliers += 1;
                    }
                    [i as f32, y, 0.]
                })
                .collect::<Vec<_>>();
            let line = [line, (0..nb_init_inliers).map(|i| [i as f32, 0., 0.]).collect::<Vec<_>>()].concat();
            let parameter = SacAlgorithmParameter {
                probability: 0.99, 
                threshold: 0.1, 
                max_iterations: 2000,
                threads: 1
            };
            let mut model = SacModelLine::with_data(&line);
            let mut algorithm = SacRansac{
                parameter,
                inliers: vec![],
            };
            let result = algorithm.compute(&mut model);
            let _factor = model.get_coefficient();
            println!("Factor: {:?}", _factor);
            // let inliers = &algorithm.inliers;
            let inliers = algorithm.get_inliers();

            assert!(result);

            let error_factor = 0.05f32;
            let nb_error = inliers.len() - nb_inliers;
            println!("Real Error factor: {}, Target factor: {error_factor}", (nb_error as f32) / (nb_inliers as f32));
            assert!((nb_error as f32) < (nb_inliers as f32 * error_factor));
        }

        #[test]
        fn sac_line_3() {
            use rand::Rng;
            let coefficient = ([0f32, 0., 0.], [1f32, 0., 0. ]);
            let mut rng = rand::thread_rng();
            for _ in 0..1000 {
                let y = rng.gen_range(0f32..=1.);
                let distance = SacModelLine::compute_point_to_model([0., y, 0.], &coefficient);
                let diff = ((distance - y) * 10000.).round();
                assert!(diff <= 0.);
            }
        }
    }

    mod circle3d {
        use f3l_core::{apply_both, round_n, round_slice_n, SimpleSliceMath};
        use f3l_core::glam::{Mat3, Mat4, Vec4, Vec3, Vec3A, Quat};
        use super::*;

        #[test]
        fn test_circle_coefficients() {
            let c = [0f32, 0f32, 0f32];
            let r = 4.5f32;
            let degrees = [20f32, 120., -100.];
            let point = degrees.into_iter()
                .map(|d| {
                    let rad = d.to_radians();
                    [
                        c[0] + rad.sin() * r,
                        c[1] + rad.cos() * r,
                        c[2],
                    ].into()
                }).collect::<Vec<Vec3A>>();

            let mat = Mat3::from_euler(f3l_core::glam::EulerRot::XYZ, 20f32.to_radians(), -40f32.to_radians(), 65f32.to_radians());
            let mat4 = Mat4::from_rotation_translation(Quat::from_mat3(&mat), Vec3::new(10f32, -20f32, 30f32));

            let point = point.iter()
                .map(|&v| (mat4 * Vec4::from((v, 1f32))).into()).collect::<Vec<Vec3A>>();
            let c = mat4.project_point3(c.into());
            let n = mat * Vec3A::Z;
            let n = n.normalize();
            
            let [p1, p2, p3] = [point[0].into(), point[1].into(), point[2].into()];

            // Copy From SacModelCircle3d::compute_model_coefficients
            {
                let v12 = apply_both(&p1, &p2, std::ops::Sub::sub);
                let v21 = apply_both(&p2, &p1, std::ops::Sub::sub);
                let v13 = apply_both(&p1, &p3, std::ops::Sub::sub);
                let v31 = apply_both(&p3, &p1, std::ops::Sub::sub);
                let v23 = apply_both(&p2, &p3, std::ops::Sub::sub);
                let v32 = apply_both(&p3, &p2, std::ops::Sub::sub);
                
                let normal = v12.cross(&v23);
                let common_divided = 1f32 / (2. * normal.len().powi(2));

                let alpha = (v23.len().powi(2) * v12.dot(&v13)) * common_divided;
                let beta = (v13.len().powi(2) * v21.dot(&v23)) * common_divided;
                let gamma = (v12.len().powi(2) * v31.dot(&v32)) * common_divided;

                let mut pc = [0.; 3];
                (0..3).for_each(|i| pc[i] = alpha * p1[i] + beta * p2[i] + gamma * p3[i]);

                let normal = normal.normalized();
                let angle_normals = normal.compute_angle(&n.into()).to_degrees().round();

                let radius = apply_both(&pc, &p1, std::ops::Sub::sub).len();

                assert_eq!(round_slice_n(c.into(), 4), round_slice_n(pc, 4));
                assert_eq!(round_n(r, 4), round_n(radius, 4));
                assert!(
                    angle_normals== 0f32 || angle_normals==180f32
                );
            }
        }

        #[test]
        fn distance_between_point_circle() {
            let mat = Mat3::from_euler(f3l_core::glam::EulerRot::XYZ, 20f32.to_radians(), -40f32.to_radians(), 65f32.to_radians());
            let mat4 = Mat4::from_rotation_translation(Quat::from_mat3(&mat), Vec3::new(10f32, -20f32, 30f32));

            let project_point = Vec3::new(3., 0., 5.);
            // target_distance: 1) to plane, 2) plane to round
            // let target_distance = 1.5f32;
            let target_distance = project_point.distance(Vec3::new(4.5, 0., 0.));
            let project_point = mat4.project_point3(project_point);

            let c = [0f32, 0f32, 0f32];
            let r = 4.5f32;
            let c = mat4.project_point3(c.into());
            let n = mat * Vec3A::Z;
            let n = n.normalize();

            let coefficients = (c.into(), n.into(), r);
            let distance = SacModelCircle3d::compute_point_to_model(project_point, &coefficients);
            println!("Projected: {:?}", project_point);
            assert_eq!(round_n(target_distance, 4), round_n(distance, 4));
        }
    }

    mod sphere {
        use super::*;
        use f3l_core::{round_n, round_slice_n};
        use f3l_core::glam::Vec3A;

        #[test]
        fn sphere_coefficients() {
            let center = Vec3A::new(10., -20., 30.);
            let center_slice: [f32; 3] = center.into();
            let radius = 35f32;

            let data = (0..10).flat_map(|i| {
                let theta = ((i as f32) * 36.).to_radians();
                (0..10).map(|ii| {
                    let phi = ((ii as f32) * 36.).to_radians();
                    center + radius * Vec3A::new(
                        theta.sin() * phi.cos(),
                        theta.sin() * phi.sin(),
                        theta.cos()
                    )
                }).collect::<Vec<_>>()
            }).collect::<Vec<_>>();

            let parameter = SacAlgorithmParameter {
                threshold: 1f32,
                ..Default::default()
            };
            let mut model = SacModelSphere::with_data(&data);
            let mut algorithm = SacRansac{parameter, inliers: vec![]};
            let ok = algorithm.compute(&mut model);

            assert!(ok);
            let (center, r) = model.get_coefficient();

            assert_eq!(round_n(r, 4), radius);
            assert_eq!(center_slice, round_slice_n(center, 4));
            
        }

    }
}