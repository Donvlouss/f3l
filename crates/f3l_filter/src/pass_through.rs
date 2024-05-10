use std::ops::{Bound, Range};

use crate::F3lFilterInverse;

use super::F3lFilter;
use f3l_core::{serde::{self, Serialize, Deserialize}, BasicFloat};

/// Target `Dimension` to filter with `Upper-Bound` and `Lower-Bound`
///
/// # Examples
/// ```
/// let vertices = load_ply("../../data/table_scene_lms400.ply");
///
/// let mut filter = PassThrough::with_data(
///     &vertices,
///     Range {
///         start: Bound::Included(0.),
///         end: Bound::Included(0.5),
///     },
///     0,
/// );
/// let start = Instant::now();
///
/// let out = filter.filter_instance();
/// ```
///
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate="self::serde")]
pub struct PassThrough<T: BasicFloat>
{
    pub negative: bool,
    pub dim: usize,
    pub bound: Option<Range<Bound<T>>>,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    inliers: Vec<usize>,
}

impl<T: BasicFloat> Default for PassThrough<T>
{
    fn default() -> Self {
        Self {
            negative: false,
            dim: Default::default(),
            bound: Default::default(),
            inliers: Default::default(),
        }
    }
}

impl<T: BasicFloat> PassThrough<T>
{
    pub fn with_data(bound: Range<Bound<T>>, dim: usize) -> Self {
        Self {
            negative: false,
            dim,
            bound: Some(bound),
            inliers: Default::default(),
        }
    }

    pub fn set_parameter(&mut self, dim: usize, bound: Range<Bound<T>>) {
        self.dim = dim;
        self.bound = Some(bound);
    }
}

impl<T: BasicFloat> F3lFilterInverse for PassThrough<T> {

    fn set_negative(&mut self, negative: bool) {
        self.negative = negative;
    }
}

impl<'a, P, T: BasicFloat, const D: usize> F3lFilter<'a, P, D> for PassThrough<T>
where
    P: Into<[T; D]> + Clone + Copy + Send + Sync,
    [T; D]: Into<P>,
{
    fn filter(&mut self, data: &'a Vec<P>) -> Vec<usize> {
        if !self.apply_filter(data) {
            return vec![];
        }
        self.inliers.clone()
    }

    fn filter_instance(&mut self, data: &'a Vec<P>) -> Vec<P> {
        if !self.apply_filter(data) {
            return vec![];
        }
        // let data = self.data.unwrap();
        self.inliers.iter().map(|i| data[*i]).collect()
    }

    fn apply_filter(&mut self, data: &'a Vec<P>) -> bool {
        if self.dim >= D {
            return false;
        }

        let (start, end) = if let Some(bound) = &self.bound {
            (bound.start, bound.end)
        } else {
            return false;
        };

        use f3l_core::rayon::prelude::*;
        self.inliers = data
            .par_iter()
            .enumerate()
            .filter_map(|(i, &p)| {
                let p: [T; D] = p.into();
                let p = p[self.dim];
                let b_start = match start {
                    Bound::Included(v) => p >= v,
                    Bound::Excluded(v) => p > v,
                    Bound::Unbounded => true,
                };
                let b_end = match end {
                    Bound::Included(v) => p <= v,
                    Bound::Excluded(v) => p < v,
                    Bound::Unbounded => true,
                };
                if (!self.negative && (b_start && b_end)) || (self.negative && !(b_start && b_end))
                {
                    Some(i)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        true
    }
}

#[test]
fn serde() {
    let model = PassThrough {
        negative: false,
        dim: 0,
        bound: Some(Range {
            start: Bound::Included(0f32),
            end: Bound::Included(0.5),
        },),
        inliers: vec![],
    };
    let content = serde_json::to_string(&model).unwrap();
    println!("{}", content);
    let text = r#"{
        "negative":false,
        "dim":0,
        "bound":{
            "start":{"Included":0.0},
            "end":{"Included":0.5}
        }}"#;
    let model_de: PassThrough<f32> = serde_json::from_str(text).unwrap();
    assert_eq!(model.negative, model_de.negative);
    assert_eq!(model.dim, model_de.dim);
    assert_eq!(model.bound, model_de.bound);
    assert_eq!(model.inliers, model_de.inliers);

}