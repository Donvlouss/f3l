use std::ops::{Bound, Range};

use crate::F3lFilterInverse;

use super::F3lFilter;
use f3l_core::{serde::{self, Serialize, Deserialize}, BasicFloat};

type DirectionRange<T> = Vec<(usize, Range<Bound<T>>)>;

/// A `Dimension-wise` to filter with `Upper-Bound` and `Lower-Bound`
///
/// # Examples
/// ```
/// let vertices = load_ply("../../data/table_scene_lms400.ply");
///
/// let range = vec![
///     (0, Bound::Included(-0.3)..Bound::Included(0.5)),
///     (1, Bound::Included(0.)..Bound::Included(0.8)),
///     (2, Bound::Included(-1.4)..Bound::Included(-1.3)),
/// ];
/// let mut filter = ConditionRemoval::with_data(&vertices, &range);
///
/// let out = filter.filter_instance();
/// ```
///
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate="self::serde")]
pub struct ConditionRemoval<T: BasicFloat>
{
    pub negative: bool,
    pub bound: DirectionRange<T>,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    inliers: Vec<usize>,
}

impl<T: BasicFloat> Default for ConditionRemoval<T>
{
    fn default() -> Self {
        Self {
            negative: false,
            bound: Default::default(),
            inliers: Default::default(),
        }
    }
}

impl<T: BasicFloat> ConditionRemoval<T>
{
    // pub fn with_data(data: &'a Vec<P>, bound: &'a Vec<(usize, Bound<T>, Bound<T>)>) -> Self {
    pub fn with_data(bound: &DirectionRange<T>) -> Self {
        Self {
            negative: false,
            bound: bound.clone(),
            inliers: Default::default(),
        }
    }

    pub fn set_parameter(&mut self, bound: &DirectionRange<T>) {
        self.bound = bound.clone();
    }
}

impl<T: BasicFloat> F3lFilterInverse for ConditionRemoval<T> {

    fn set_negative(&mut self, negative: bool) {
        self.negative = negative;
    }
}

impl<'a, P, T: BasicFloat, const D: usize> F3lFilter<'a, P, D> for ConditionRemoval<T>
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

        use f3l_core::rayon::prelude::*;
        self.inliers = data
            .par_iter()
            .enumerate()
            .filter_map(|(i, &p)| {
                let p: [T; D] = p.into();
                let mut ok = true;
                self.bound.iter().for_each(|(dim, bound)| {
                    let p = p[*dim];
                    let b_start = match bound.start {
                        Bound::Included(v) => p >= v,
                        Bound::Excluded(v) => p > v,
                        Bound::Unbounded => true,
                    };
                    let b_end = match bound.end {
                        Bound::Included(v) => p <= v,
                        Bound::Excluded(v) => p < v,
                        Bound::Unbounded => true,
                    };
                    if (!self.negative && (b_start && b_end))
                        || (self.negative && !(b_start && b_end))
                    {
                        ok &= true;
                    } else {
                        ok = false;
                    }
                });
                if ok {
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
    let range = vec![
        (0_usize, Bound::Included(-0.3)..Bound::Included(0.5f32)),
        (1, Bound::Included(0.)..Bound::Included(0.8)),
        (2, Bound::Included(-1.4)..Bound::Included(-1.3)),
    ];
    let model = ConditionRemoval {
        negative: false,
        inliers: vec![],
        bound: range,
    };
    let content = serde_json::to_string(&model).unwrap();
    println!("{}", content);

    let text = r#"{
        "negative":false,
        "bound":[
            [0,{"start":{"Included":-0.3},"end":{"Included":0.5}}],
            [1,{"start":{"Included":0.0},"end":{"Included":0.8}}],
            [2,{"start":{"Included":-1.4},"end":{"Included":-1.3}}]
            ]
        }"#;
    let model_de: ConditionRemoval<f32> = serde_json::from_str(text).unwrap();
    assert_eq!(model.negative, model_de.negative);
    assert_eq!(model.bound, model_de.bound);
    assert_eq!(model.inliers, model_de.inliers);
}