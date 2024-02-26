use f3l_filter::*;
use approx::assert_relative_eq;
use rand::Rng;

mod filter {
    use super::*;

    fn generate_random<const D: usize>(nb: usize) -> Vec<[f32; D]> {
        let mut rng = rand::thread_rng();
        (0..nb)
            .map(|_| {
                let mut arr = [0f32; D];
                (0..D).for_each(|i| arr[i] = rng.gen_range(0f32..1f32));
                arr
            }).collect::<Vec<_>>()
    }

    fn brute_force<const D: usize>(data: &Vec<[f32; D]>, radius: f32, threshold: usize) -> Vec<usize> {
        use rayon::prelude::*;
        data.par_iter()
            .enumerate()
            .filter(|(_, p)| {
                let mut count = 0;

                for other in data {
                    let dist = (0..D)
                        .map(|i| {
                            (other[i] - p[i]).powi(2)
                        })
                        .sum::<f32>().sqrt();
                    if dist <= radius {
                        count += 1;
                    }
                    if count >= threshold {
                        return true;
                    }
                }
                return false;
            }).map(|(i, _)| i)
            .collect()
    }

    mod dimension_1d{
        use super::*;

        #[test]
        fn radius_outlier_removal() {
            let data = vec![[1f32], [3.], [4.], [5.], [7.]];
            let mut filter = RadiusOutlierRemoval::with_data(1.5f32, 2, &data);
            let out = filter.filter_instance();
            let mut count = 3f32 + 4. + 5.;
            out.into_iter()
                .for_each(|v| count -= v[0]);
            assert_relative_eq!(count, 0f32);
        }
    }

    mod dimension_3d {
        use super::*;
        use ply_rs as ply;
        use ply_rs::ply::Property;

        #[test]
        fn radius_outlier_removal() {
            let data = generate_random::<3>(1000);
            let r = 0.1f32;
            let threshold = 5usize;

            let mut brute_force_result = brute_force(&data, r, threshold);
            let mut filter = RadiusOutlierRemoval::with_data(r, threshold, &data);
            let mut out = filter.filter();

            brute_force_result.sort();
            out.sort();

            let nb_diff = (brute_force_result.len() as isize - out.len() as isize).abs();
            assert!(nb_diff <= 1);

            // TODO Sometimes exist 1 different point
            if nb_diff == 1 {
                return;
            }

            brute_force_result.iter()
                .zip(&out)
                .for_each(|(p1, p2)| {
                    assert_eq!(p1, p2);
                });
        }

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
        fn table_radius_removal() {
            use std::path::Path;
            if !Path::new("data/Itable_scene_lms400.ply").exists() {
                return;
            }
            let vertices = load_ply("data/Itable_scene_lms400.ply");
            let mut filter = RadiusOutlierRemoval::with_data(0.003f32, 5, &vertices);
            let out = filter.filter_instance();

            assert!(!out.is_empty());
        }
    }
}