use std::ops::Index;

use f3l_core::rayon::prelude::*;
use f3l_core::{
    serde::{self, Deserialize, Serialize},
    BasicFloat,
};
use f3l_search_tree::{KdTree, SearchBy, TreeRadiusResult, TreeResult};

use crate::{F3lFilter, F3lFilterInverse};

/// Filter Numbers of point in radius.
///
/// # Examples
/// ```
/// let vertices = load_ply("../../data/table_scene_lms400.ply");
/// let mut filter = RadiusOutlierRemoval::with_data(0.03f32, 20);
/// // set true to get outlier instead of inliers
/// //filter.set_negative(true);
/// let out = filter.filter_instance(&vertices);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "self::serde")]
pub struct RadiusOutlierRemoval<'a, P, T: BasicFloat, const D: usize>
where
    P: Into<[T; D]> + Clone + Copy + Index<usize, Output = T>,
    [T; D]: Into<P>,
{
    pub negative: bool,
    pub radius: T,
    pub threshold: usize,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    tree: KdTree<'a, T, P>,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    inlier: Vec<bool>,
}

impl<'a, P, T: BasicFloat, const D: usize> RadiusOutlierRemoval<'a, P, T, D>
where
    P: Into<[T; D]> + Clone + Copy + Send + Sync + Index<usize, Output = T>,
    [T; D]: Into<P>,
{
    pub fn new(radius: T, threshold: usize) -> Self {
        Self {
            negative: false,
            radius,
            threshold,
            tree: KdTree::<T, P>::new(D),
            inlier: vec![],
        }
    }

    #[inline]
    fn ok(&self, is_inlier: bool) -> bool {
        (!is_inlier && self.negative) || (is_inlier && !self.negative)
    }
}

impl<'a, P, T: BasicFloat, const D: usize> F3lFilterInverse for RadiusOutlierRemoval<'a, P, T, D>
where
    P: Into<[T; D]> + Clone + Copy + Index<usize, Output = T>,
    [T; D]: Into<P>,
{
    fn set_negative(&mut self, negative: bool) {
        self.negative = negative;
    }
}

impl<'a, P, T: BasicFloat, const D: usize> F3lFilter<'a, P, D> for RadiusOutlierRemoval<'a, P, T, D>
where
    P: Into<[T; D]> + Clone + Copy + Send + Sync + Index<usize, Output = T>,
    [T; D]: Into<P>,
{
    fn filter(&mut self, data: &'a [P]) -> Vec<usize> {
        self.apply_filter(data);

        self.inlier
            .iter()
            .enumerate()
            .filter(|&(_, f)| self.ok(*f))
            .map(|(i, _)| i)
            .collect()
    }

    fn filter_instance(&mut self, data: &'a [P]) -> Vec<P> {
        self.apply_filter(data);

        self.inlier
            .iter()
            .enumerate()
            .filter(|&(_, f)| self.ok(*f))
            .map(|(i, _)| data[i])
            .collect()
    }

    fn apply_filter(&mut self, data: &'a [P]) -> bool {
        if data.is_empty() {
            return false;
        }
        // Check Tree dimension correct, cause skip deserialize would be 0.
        if self.tree.dim != D {
            self.tree = KdTree::<T, P>::new(D);
        }
        self.tree.set_data(data);

        self.tree.build();
        let capacity = data.len() / 10;
        let capacity = if capacity > 10 { capacity } else { 10 };

        let r = (self.radius.to_f32().unwrap()).powi(2);
        let by = if self.radius == T::zero() {
            SearchBy::Radius(1.0)
        } else {
            SearchBy::Radius(r)
        };

        let th = self.threshold;

        self.inlier.resize(data.len(), false);
        let inlier = data
            .par_iter()
            .enumerate()
            .map(|(i, p)| {
                let mut result =
                    TreeRadiusResult::with_capacity(r, capacity).set_to_maximum_size(th);
                self.tree.search(*p, by, &mut result);
                (i, result.data.len() >= th)
            })
            .collect::<Vec<_>>();
        inlier
            .iter()
            .filter(|(_, f)| *f)
            .for_each(|(i, _)| self.inlier[*i] = true);

        true
    }
}

#[test]
fn serde() {
    let model = RadiusOutlierRemoval::<[f32; 3], f32, 3>::new(0.03f32, 20_usize);
    let content = serde_json::to_string(&model).unwrap();
    println!("{}", content);

    let text = r#"{
        "negative":false,
        "radius":0.03,
        "threshold":20
    }"#;
    let model_de: RadiusOutlierRemoval<[f32; 3], f32, 3> = serde_json::from_str(text).unwrap();
    assert_eq!(model.negative, model_de.negative);
    assert_eq!(model.radius, model_de.radius);
    assert_eq!(model.threshold, model_de.threshold);
}
