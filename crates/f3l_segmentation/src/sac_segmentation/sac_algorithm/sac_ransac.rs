use std::sync::{Arc, Mutex};

use f3l_core::BasicFloat;

use super::{SacAlgorithm, SacAlgorithmGetter, SacAlgorithmParameter};
use crate::sac_model::SacModel;

#[derive(Debug, Default, Clone)]
pub struct SacRansac {
    pub parameter: SacAlgorithmParameter,
    pub inliers: Vec<usize>,
}

impl SacAlgorithmGetter for SacRansac {
    fn get_inliers(&self) -> &Vec<usize> {
        &self.inliers
    }
}

impl<'a, P: Copy, T, R> SacAlgorithm<'a, P, T, R> for SacRansac
where
    T: BasicFloat,
    R: SacModel<'a, P, T> + Send + Sync,
    <R as SacModel<'a, P, T>>::CoefficientsType: Send + Sync,
{
    fn compute(&mut self, model: &mut R) -> bool {
        let SacAlgorithmParameter {
            probability,
            threshold,
            max_iterations,
            threads,
        } = self.parameter;

        let max_skip = max_iterations * 100;

        let skipped = Arc::new(Mutex::new(0usize));
        let iterations = Arc::new(Mutex::new(0usize));
        let nb_best_inliers = Arc::new(Mutex::new(0usize));

        let nb_data = model.data_len();
        let log_probability = (1. - probability).ln();
        let one_over_indices = 1. / nb_data as f32;
        let nb_sample = R::NB_SAMPLE;

        let coefficient = model.get_coefficient();
        let coefficient = Arc::new(Mutex::new(coefficient));

        let closure = || {
            loop {
                {
                    let lock = iterations.lock().unwrap();
                    if *lock > max_iterations {
                        break;
                    }
                    let lock = skipped.lock().unwrap();
                    if *lock > max_skip {
                        // return;
                        break;
                    }
                }
                let samples = model.get_random_samples();

                let result = model.compute_model_coefficients(&samples);
                if result.is_err() {
                    let mut lock = skipped.lock().unwrap();
                    *lock += 1;
                    break;
                }
                let result = result.unwrap();
                let nb_inlier =
                    model.count_indices_within_tolerance(&result, T::from(threshold).unwrap());

                let mut k: f32 = 0f32;
                {
                    let mut lock = nb_best_inliers.lock().unwrap();
                    if nb_inlier > *lock {
                        *lock = nb_inlier;

                        let mut lock = coefficient.lock().unwrap();
                        *lock = result;
                        let w = nb_inlier as f32 * one_over_indices;
                        let mut p_outlier = 1. - w.powi(nb_sample as i32);
                        p_outlier = p_outlier.max(f32::EPSILON);
                        p_outlier = p_outlier.min(1. - f32::EPSILON);
                        k = log_probability / p_outlier.ln();
                    }
                }
                {
                    let mut lock = iterations.lock().unwrap();
                    *lock += 1;

                    if k > 1. && *lock as f32 > k {
                        break;
                    }
                }
            }
        };

        if threads > 1 {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build()
                .unwrap();
            pool.in_place_scope(|s| {
                for _ in 0..threads {
                    s.spawn(|_| closure());
                }
            });
        } else {
            closure();
        }

        let coefficient = coefficient.lock().unwrap();
        model.set_coefficient(&coefficient);
        self.inliers =
            model.select_indices_within_tolerance(&coefficient, T::from(threshold).unwrap());

        true
    }
}
