use f3l_filter::*;
use std::ops::Bound;

mod filter {
    use super::*;

    mod dimension_1d {

        use super::*;

        #[test]
        fn pass_through_1d() {
            let data = (0..10usize)
                .map(|i| [i as f32]).collect::<Vec<_>>();
            let mut filter = PassThrough::with_data(
                &data, Bound::Included(0.)..Bound::Excluded(6.), 0
            );
            let out = filter.filter();
            assert_eq!(out.len(), 6);
        }

        #[test]
        fn pass_through_1d_negative() {
            let data = (0..10usize)
                .map(|i| [i as f32]).collect::<Vec<_>>();
            let mut filter = PassThrough::with_data(
                &data, Bound::Included(0.)..Bound::Excluded(6.), 0
            );
            filter.set_negative(true);
            let out = filter.filter();
            assert_eq!(out.len(), 4);
        }
    }

    mod dimension_3d {
        use super::*;
        use glam::Vec3;

        #[test]
        fn pass_through_3d() {
            let data = 
                (0..100).flat_map(|x| {
                    (0..100).flat_map(|y| {
                        (0..100).map(|z| {
                                Vec3::new(x as f32, y as f32, z as f32)
                            })
                            .collect::<Vec<_>>()
                    }).collect::<Vec<_>>()
                }).collect::<Vec<_>>();
            
            let size = 100 * 100 * 20usize;

            let mut filter = PassThrough::with_data(
                &data, Bound::Included(80.)..Bound::Unbounded, 2);
            let out = filter.filter();

            assert_eq!(out.len(), size);
        }
    }
}