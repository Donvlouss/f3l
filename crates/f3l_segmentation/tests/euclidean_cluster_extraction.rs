use f3l_segmentation::*;

mod segmentation {
    use super::*;

    mod dimension_1d {
        use super::*;

        #[test]
        fn test_euclidean_cluster_1d() {
            let data = vec![[0.], [1.], [4.], [5.], [6.], [9f32]];
            let parameter = F3lClusterParameter {
                tolerance: 2f32,
                nb_in_tolerance: 1,
                min_nb_data: 1,
                max_nb_data: 10,
                max_nb_cluster: 10,
            };

            let mut extractor = EuclideanClusterExtractor::with_data(
                parameter, &data
            );
            let cluster = extractor.extract();
            assert_eq!(cluster.len(), 3);
        }
    }

    mod dimension_3d {
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
        fn test_euclidean_cluster_3d() {
            use std::path::Path;
            if !Path::new("data/table_remove_plane.ply").exists() {
                return;
            }
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
            assert_eq!(clusters.len(), 5);
        }
    }

}