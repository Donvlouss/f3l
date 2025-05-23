mod kd_tree {
    use approx::assert_relative_eq;
    #[cfg(all(feature = "core", not(feature = "pure")))]
    use f3l_core::glam::{Vec2, Vec3};
    use f3l_search_tree::*;
    use nalgebra::Point3;

    mod insert {
        use std::ops::Index;

        use super::*;

        #[cfg(all(feature = "core", not(feature = "pure")))]
        #[test]
        pub fn test_insert_data_glam() {
            let data = vec![Vec3::new(1.0, 2.0, 3.0), Vec3::new(4.0, 5.0, 6.0)];
            let tree = KdTree::with_data(3, &data);

            let mut d = 1.0f32;
            tree.data.unwrap().iter().for_each(|element| {
                (0..3).for_each(|i| {
                    assert_relative_eq!(d, element[i]);
                    d += 1f32;
                });
            });
        }
        #[test]
        pub fn test_insert_data_nalgebra() {
            let data = vec![
                Point3::<f32>::new(1.0, 2.0, 3.0),
                Point3::<f32>::new(4.0, 5.0, 6.0),
            ];
            let tree = KdTree::with_data(3, &data);

            let mut d = 1.0f32;
            tree.data.unwrap().iter().for_each(|element| {
                (0..3).for_each(|i| {
                    assert_relative_eq!(d, element[i]);
                    d += 1f32;
                });
            });
        }

        #[derive(Debug, Clone, Copy)]
        struct MyStruct {
            x: f32,
            y: f32,
            z: f32,
        }

        impl From<[f32; 3]> for MyStruct {
            fn from(value: [f32; 3]) -> Self {
                MyStruct {
                    x: value[0],
                    y: value[1],
                    z: value[2],
                }
            }
        }

        impl From<MyStruct> for [f32; 3] {
            fn from(value: MyStruct) -> Self {
                [value.x, value.y, value.z]
            }
        }

        impl Index<usize> for MyStruct {
            type Output = f32;

            fn index(&self, index: usize) -> &Self::Output {
                match index {
                    0 => &self.x,
                    1 => &self.y,
                    2 => &self.z,
                    _ => panic!("Out of Range"),
                }
            }
        }

        #[test]
        pub fn test_insert_data_custom() {
            let data = vec![
                MyStruct {
                    x: 1.0f32,
                    y: 2.0f32,
                    z: 3.0f32,
                },
                MyStruct {
                    x: 4.0f32,
                    y: 5.0f32,
                    z: 6.0f32,
                },
            ];
            let tree = KdTree::with_data(3, &data);

            let mut d = 1.0f32;
            tree.data.unwrap().iter().for_each(|element| {
                (0..3).for_each(|i| {
                    assert_relative_eq!(d, element[i]);
                    d += 1f32;
                });
            });
        }

        #[test]
        pub fn test_insert_data_array() {
            let data = vec![[1f32, 2f32, 3f32], [4f32, 5f32, 6f32]];
            let tree = KdTree::with_data(3, &data);

            let mut d = 1.0f32;
            tree.data.unwrap().iter().for_each(|element| {
                (0..3).for_each(|i| {
                    assert_relative_eq!(d, element[i]);
                    d += 1f32;
                });
            });
        }
    }

    mod query {
        use super::*;

        mod dimension_1 {
            use super::*;
            #[test]
            fn query_nearest_knn_1d() {
                let data = (0..10).map(|i| [i as f32]).collect::<Vec<_>>();
                let mut tree = KdTree::with_data(1, &data);
                tree.build();
                let result = tree.search_knn(&[5.1f32], 1);
                let nearest_data = result[0].0[0];
                let nearest_distance = result[0].1;

                assert_relative_eq!(nearest_data, 5f32);
                assert_relative_eq!(nearest_distance, 0.1f32);
            }

            #[test]
            fn query_knn_4_1d() {
                let data = (0..10).map(|i| [i as f32]).collect::<Vec<_>>();
                let mut tree = KdTree::with_data(1, &data);
                tree.build();
                // range: 3.1 to 7.1
                let result = tree.search_knn(&[5.1f32], 4);

                let mut count = 4f32 + 5. + 6. + 7.;
                result.into_iter().for_each(|(p, _)| {
                    count -= p[0];
                });
                assert_relative_eq!(count, 0f32);
            }

            #[test]
            fn query_radius_2_1d() {
                let data = (0..10).map(|i| [i as f32]).collect::<Vec<_>>();
                let mut tree = KdTree::with_data(1, &data);
                tree.build();
                // range: 3.1 to 7.1
                let result = tree.search_radius(&[5.1f32], 2f32);

                let mut count = 4f32 + 5. + 6. + 7.;
                result.into_iter().for_each(|p| {
                    count -= p[0];
                });
                assert_relative_eq!(count, 0f32);
            }
        }

        #[cfg(all(feature = "core", not(feature = "pure")))]
        mod dimension_2 {
            use super::*;

            #[test]
            fn query_nearest_knn_glam_2d() {
                let data = (0..10)
                    .flat_map(|y| {
                        (0..10)
                            .map(|x| Vec2::new(x as f32, y as f32))
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>();
                let mut tree = KdTree::with_data(2, &data);
                tree.build();
                let target = Vec2::new(5.1, 5.1);

                let result = tree.search_knn(&target, 1);
                let nearest_data = result[0].0;
                let nearest_distance = result[0].1;

                assert_relative_eq!(nearest_data.distance(Vec2::new(5f32, 5f32)), 0f32);
                assert_relative_eq!(
                    (nearest_distance * 1000000f32).round(),
                    ((0.1f32.powi(2) * 2.).sqrt() * 1000000f32).round()
                );
            }
        }

        #[cfg(all(feature = "core", not(feature = "pure")))]
        mod dimension_3 {
            use super::*;

            #[test]
            fn query_nearest_knn_nalgebra_3d() {
                let data = (0..10)
                    .flat_map(|x| {
                        (0..10)
                            .flat_map(|y| {
                                (0..10)
                                    .map(|z| Vec3::new(x as f32, y as f32, z as f32))
                                    .collect::<Vec<_>>()
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>();
                let mut tree = KdTree::with_data(3, &data);
                tree.build();

                let target = Vec3::new(5.1, 5.1, 5.1);
                let result = tree.search_knn(&target, 1);
                let nearest_data = result[0].0;
                let nearest_distance = result[0].1;

                let distance: f32 = (0..3).map(|i| nearest_data[i] - 5f32).sum();
                assert_relative_eq!(distance, 0f32);
                assert_relative_eq!(
                    (nearest_distance * 1000000f32).round(),
                    ((0.1f32.powi(2) * 3.).sqrt() * 1000000f32).round()
                );
            }
        }
    }
}
