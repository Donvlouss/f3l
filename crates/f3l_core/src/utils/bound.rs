use crate::traits::BasicFloat;

pub fn get_minmax<P, T: BasicFloat, const D: usize>(cloud: &[P]) -> (P, P)
where
    P: Into<[T; D]> + Clone + Copy,
    [T; D]: Into<P>,
{
    assert!(!cloud.is_empty());

    let mut min: [T; D] = cloud[0].into();
    let mut max: [T; D] = cloud[0].into();

    cloud.iter().for_each(|v| {
        let pt: [T; D] = (*v).into();
        (0..D).for_each(|i| {
            if pt[i] < min[i] {
                min[i] = pt[i];
            }
            if pt[i] > max[i] {
                max[i] = pt[i];
            }
        });
    });
    (min.into(), max.into())
}
