use f3l_filter::*;

mod filter {
    use super::*;

    mod dimension_1d {
        use super::*;

        #[test]
        fn voxel_grid_1d() {
            let data = (0..10)
                .map(|i| [i as f32]).collect::<Vec<_>>();
            let mut filter = VoxelGrid::with_data(&data, &[2f32]);
            let out = filter.filter_instance();

            assert_eq!(out.len(), 5);
            (0..5)
                .for_each(|i| {
                    let d = i as f32 * 2. + 0.5;
                    assert!(out.contains(&[d]));
                });
        }
    }

    mod dimension_3d {
        use super::*;
        use f3l_core::glam::Vec3;

        #[test]
        fn voxel_grid_3d() {
            let data = 
                (0..100).flat_map(|x| {
                    (0..100).flat_map(|y| {
                        (0..100).map(|z| {
                                Vec3::new(x as f32, y as f32, z as f32)
                            })
                            .collect::<Vec<_>>()
                    }).collect::<Vec<_>>()
                }).collect::<Vec<_>>();
            let leaf = [5f32; 3];
            let size = (100f32 / 5.0).powi(3) as usize;
            
            let mut filter = VoxelGrid::with_data(&data, &leaf.into());
            let out = filter.filter_instance();

            assert_eq!(size, out.len());
        }
    }
}