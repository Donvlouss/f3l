use f3l_filter::*;
use std::ops::Bound;

mod filter {
    use super::*;

    mod dimension_1d {

        use super::*;

        #[test]
        fn condition_removal_1d() {
            let data = (0..10usize).map(|i| [i as f32]).collect::<Vec<_>>();
            let bound = vec![(0, (Bound::Included(0.)..Bound::Excluded(6.)))];
            let mut filter = ConditionRemoval::with_data(&bound);
            let out = filter.filter(&data);
            assert_eq!(out.len(), 6);
        }

        #[test]
        fn condition_removal_1d_negative() {
            let data = (0..10usize).map(|i| [i as f32]).collect::<Vec<_>>();
            let bound = vec![
                (0, (Bound::Included(0.)..Bound::Excluded(6.))),
                (0, (Bound::Included(4.)..Bound::Unbounded)),
            ];
            let mut filter = ConditionRemoval::with_data(&bound);
            let out = filter.filter(&data);
            assert_eq!(out.len(), 2);
        }
    }

    mod dimension_3d {
        use super::*;
        use f3l_core::glam::Vec3;

        #[test]
        fn condition_removal_3d() {
            let data = (0..100)
                .flat_map(|x| {
                    (0..100)
                        .flat_map(|y| {
                            (0..100)
                                .map(|z| Vec3::new(x as f32, y as f32, z as f32))
                                .collect::<Vec<_>>()
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();

            let size = 20 * 30 * 40usize;
            let range = vec![
                (0, Bound::Included(40.)..Bound::Excluded(60.)),
                (1, Bound::Unbounded..Bound::Excluded(30.)),
                (2, Bound::Included(60.)..Bound::Unbounded),
            ];

            let mut filter = ConditionRemoval::with_data(&range);
            let out = filter.filter(&data);

            assert_eq!(out.len(), size);
        }
    }
}
