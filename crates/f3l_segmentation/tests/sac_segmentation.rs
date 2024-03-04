use f3l_segmentation::sac_model::*;
use f3l_segmentation::sac_algorithm::*;

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
            let inliers = &algorithm.inliers;

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
                let distance = SacModelLine::compute_distance([0., y, 0.], &coefficient);
                let diff = ((distance - y) * 10000.).round();
                assert!(diff <= 0.);
            }
        }
    }
}