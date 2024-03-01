use f3l_segmentation::sac_model::*;
use f3l_segmentation::sac_algorithm::*;
use glam::Vec3;
use nalgebra::Point3;

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
                max_iterations: 200
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
}