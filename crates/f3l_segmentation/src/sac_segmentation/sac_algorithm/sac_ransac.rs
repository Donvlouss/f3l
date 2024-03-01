use std::sync::{Arc, Mutex};

use f3l_core::BasicFloat;
use rayon::prelude::*;

use super::{
    SacAlgorithm,
    SacAlgorithmParameter
};
use crate::sac_model::SacModel;

#[derive(Debug, Default, Clone)]
pub struct SacRansac {
    pub parameter: SacAlgorithmParameter,
    pub inliers: Vec<usize>
}

impl<'a, P, T, R> SacAlgorithm<'a, P, T, R> for SacRansac
where
    T: BasicFloat,
    R: SacModel<'a, P, T> + Send + Sync,
    <R as SacModel<'a, P, T>>::CoefficientsIdxType: Send + Sync
{
    fn get_inliers(&self) -> &Vec<usize> {
        &self.inliers
    }

    fn compute(&mut self, model: &mut R) -> bool {
        let SacAlgorithmParameter { probability, threshold, max_iterations }
            = self.parameter;

        let max_skip = max_iterations * 100;

        let skipped = Arc::new(Mutex::new(0usize));
        let iterations = Arc::new(Mutex::new(0usize));
        let nb_best_inliers = Arc::new(Mutex::new(0usize));

        let nb_sample = model.samples_len();
        let log_probability = (1. - probability).log10();
        let one_over_indices = 1. / nb_sample as f32;

        let coefficient = model.get_coefficient();
        let coefficient = Arc::new(Mutex::new(coefficient));
        
        // (0..max_skip)
        //     // .par_bridge()
        //     .for_each(|_i| 
        loop {
                {
                    let lock = iterations.lock().unwrap();
                    if *lock > max_iterations {
                        // return;
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
                    // return;
                    break;
                }
                let result = result.unwrap();
                let nb_inlier = model.count_indices_within_tolerance(&result, T::from(threshold).unwrap());
                
                // let mut k: f32 = 0f32;
                {
                    let mut lock = nb_best_inliers.lock().unwrap();
                    if nb_inlier > *lock {
                        *lock = nb_inlier;

                        let mut lock = coefficient.lock().unwrap();
                        *lock = result;
                        // let w = nb_inlier as f32 * one_over_indices;
                        // let mut p_no_outlier = 1. - w.powi(nb_sample as i32);
                        // p_no_outlier = p_no_outlier.max(f32::EPSILON);
                        // p_no_outlier = p_no_outlier.min(1. - f32::EPSILON);
                        // k = log_probability / p_no_outlier.log10();
                    }
                }
                // {
                    let mut lock = iterations.lock().unwrap();
                    *lock += 1;
                    
                //     if k > *lock as f32 {
                //         // return;
                //         break;
                //     }
                // }
            }
        // );

        let coefficient = coefficient.lock().unwrap();
        model.set_coefficient(&coefficient);
        self.inliers = model.select_indices_within_tolerance(&coefficient, T::from(threshold).unwrap());
        
        true
    }
}
