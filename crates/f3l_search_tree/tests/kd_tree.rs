mod kd_tree{
    use approx::assert_relative_eq;
    use f3l_core::glam::{Vec2, Vec3};
    use nalgebra::Point3;
    use f3l_search_tree::*;

    mod insert {
        use super::*;

        #[test]
        pub fn test_insert_data_glam() {
            let mut tree = KdTree::<f32, 3>::new();
            tree.set_data(&vec![
                Vec3::new(1.0, 2.0, 3.0),
                Vec3::new(4.0, 5.0, 6.0),
            ]);

            let mut d = 1.0f32;
            tree.data.iter()
                .for_each(|element| {
                    element.iter()
                        .for_each(|e| {
                            assert_relative_eq!(d, e);
                            d += 1f32;
                        })
                });
        }
        #[test]
        pub fn test_insert_data_nalgebra() {
            let mut tree = KdTree::<f32, 3>::new();
            tree.set_data(&vec![
                Point3::<f32>::new(1.0, 2.0, 3.0),
                Point3::<f32>::new(4.0, 5.0, 6.0),
            ]);

            let mut d = 1.0f32;
            tree.data.iter()
                .for_each(|element| {
                    element.iter()
                        .for_each(|e| {
                            assert_relative_eq!(d, e);
                            d += 1f32;
                        })
                });
        }

        #[derive(Debug, Clone, Copy)]
        struct MyStruct {
            x: f32,
            y: f32,
            z: f32
        }

        impl From<[f32; 3]> for MyStruct {
            fn from(value: [f32; 3]) -> Self {
                MyStruct {x: value[0], y: value[1], z: value[2]}
            }
        }

        impl From<MyStruct> for [f32; 3] {
            fn from(value: MyStruct) -> Self {
                [value.x, value.y, value.z]
            }
        }


        #[test]
        pub fn test_insert_data_custom() {
            let mut tree = KdTree::<f32, 3>::new();
            tree.set_data(&vec![
                MyStruct {x: 1.0f32, y: 2.0f32, z: 3.0f32},
                MyStruct {x: 4.0f32, y: 5.0f32, z: 6.0f32},
            ]);
            let mut d = 1.0f32;
            tree.data.iter()
                .for_each(|element| {
                    element.iter()
                        .for_each(|e| {
                            assert_relative_eq!(d, e);
                            d += 1f32;
                        })
                });
        }

        #[test]
        pub fn test_insert_data_array() {
            let mut tree = KdTree::<f32, 3>::new();
            tree.set_data(&vec![
                [1f32, 2f32, 3f32],
                [4f32, 5f32, 6f32],
            ]);

            let mut d = 1.0f32;
            tree.data.iter()
                .for_each(|element| {
                    element.iter()
                        .for_each(|e| {
                            assert_relative_eq!(d, e);
                            d += 1f32;
                        })
                });
        }
    }

    mod query {
        use super::*;

        mod dimension_1 {
            use super::*;
            #[test]
            fn query_nearest_knn_1d() {
                let mut tree = KdTree::<f32, 1>::new();
                tree.set_data(
                    &(0..10).map(|i| [i as f32]).collect()
                );
                tree.build();
                let result = tree.search_knn(&[5.1f32], 1);
                let nearest_data = result[0].0[0];
                let nearest_distance = result[0].1;

                assert_relative_eq!(nearest_data, 5f32);
                assert_relative_eq!(nearest_distance, 0.1f32);
            }

            #[test]
            fn query_knn_4_1d() {
                let mut tree = KdTree::<f32, 1>::new();
                tree.set_data(
                    &(0..10).map(|i| [i as f32]).collect()
                );
                tree.build();
                // range: 3.1 to 7.1
                let result = tree.search_knn(&[5.1f32], 4);
                
                let mut count = 4f32 + 5. + 6. + 7.;
                result.into_iter()
                    .for_each(|(p, _)| {
                        count -= p[0];
                    });
                assert_relative_eq!(count, 0f32);
            }

            #[test]
            fn query_radius_2_1d() {
                let mut tree = KdTree::<f32, 1>::new();
                tree.set_data(
                    &(0..10).map(|i| [i as f32]).collect()
                );
                tree.build();
                // range: 3.1 to 7.1
                let result = tree.search_radius(&[5.1f32], 2f32);
                
                let mut count = 4f32 + 5. + 6. + 7.;
                result.into_iter()
                    .for_each(|p| {
                        count -= p[0];
                    });
                assert_relative_eq!(count, 0f32);
            }
        }

        mod dimension_2 {
            use super::*;

            #[test]
            fn query_nearest_knn_glam_2d() {
                let mut tree = KdTree::<f32, 2>::new();
                tree.set_data(
                    &(0..10).flat_map( |y| {
                        (0..10).map(|x| {
                            Vec2::new(x as f32, y as f32)
                        }).collect::<Vec<_>>()
                    }).collect()
                );
                tree.build();
                let target = Vec2::new(5.1, 5.1);

                let result = tree.search_knn(&target, 1);
                let nearest_data = result[0].0;
                let nearest_distance = result[0].1;

                assert_relative_eq!(nearest_data.distance(Vec2::new(5f32, 5f32)), 0f32);
                assert_relative_eq!(
                    (            nearest_distance * 1000000f32).round(),
                    ((0.1f32.powi(2) * 2.).sqrt() * 1000000f32).round()
                );
            }
        }

        mod dimension_3 {
            use super::*;

            #[test]
            fn query_nearest_knn_nalgebra_3d() {
                let mut tree = KdTree::<f32, 3>::new();
                tree.set_data(
                    &(0..10).flat_map( |x| {
                        (0..10).flat_map(|y| {
                            (0..10).map(|z| {
                                Vec3::new(x as f32, y as f32, z as f32)
                            }).collect::<Vec<_>>()
                        }).collect::<Vec<_>>()
                    }).collect()
                );
                tree.build();

                let target = Point3::<f32>::new(5.1, 5.1, 5.1);
                let result = tree.search_knn(&target, 1);
                let nearest_data = result[0].0;
                let nearest_distance = result[0].1;

                let distance: f32 = (0..3)
                    .map(|i| nearest_data[i] - 5f32)
                    .sum();
                assert_relative_eq!(distance, 0f32);
                assert_relative_eq!(
                    (            nearest_distance * 1000000f32).round(),
                    ((0.1f32.powi(2) * 3.).sqrt() * 1000000f32).round()
                );
            }
        }
    }
}