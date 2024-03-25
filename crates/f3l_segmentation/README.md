# F3l Segmentation
Compute cluster or models for data.

## Cluster
### Usage
```rust
// Init parameter
let parameter = F3lClusterParameter {
    tolerance: 0.02f32,
    nb_in_tolerance: 1,
    min_nb_data: 100,
    max_nb_data: 25000,
    max_nb_cluster: 5,
};
// New and insert data.
let mut extractor = EuclideanClusterExtractor::with_data(parameter, &vertices);
// Start extracting.
let clusters = extractor.extract();
// Random color for each cluster.
let colors = (0..clusters.len())
    .map(|_| random_color())
    .collect::<Vec<_>>();
// Get points of each cluster.
let clusters = (0..clusters.len())
    .map(|i| extractor.at(i).unwrap())
    .collect::<Vec<_>>();
```

Generic Parameter of cluster method.
```rust
#[derive(Debug, Clone, Copy, Default)]
pub struct F3lClusterParameter<T: BasicFloat> {
    /// `K`-NN or `Radius` search
    pub tolerance: T,
    /// K-`NN` or `points` in Radius search
    pub nb_in_tolerance: usize,
    /// Add to clusters when numbers of cluster more than this
    pub min_nb_data: usize,
    /// Add to clusters when numbers of cluster smaller than this
    pub max_nb_data: usize,
    /// Set maximum numbers of clusters
    pub max_nb_cluster: usize,
}
```

* Euclidean Cluster
```rust
let vertices = load_ply("../../data/table_remove_plane.ply");
let parameter = F3lClusterParameter {
    tolerance: 0.02f32,
    nb_in_tolerance: 1,
    min_nb_data: 100,
    max_nb_data: 25000,
    max_nb_cluster: 5,
};
let mut extractor = EuclideanClusterExtractor::with_data(parameter, &vertices);
let clusters = extractor.extract();
let clusters = (0..clusters.len())
    .map(|i| extractor.at(i).unwrap())
    .collect::<Vec<_>>();
```
* DBScan
```rust
let vertices = load_ply("../../data/table_remove_plane.ply");

let parameter = F3lClusterParameter {
    tolerance: 0.02f32,
    nb_in_tolerance: 20,
    min_nb_data: 100,
    max_nb_data: vertices.len(),
    max_nb_cluster: 5,
};
let mut extractor = DBScan::with_data(parameter, &vertices);
let clusters = extractor.extract();
let clusters = (0..clusters.len())
    .map(|i| extractor.at(i).unwrap())
    .collect::<Vec<_>>();
```

## Segmentation
Find `Plane`, `Sphere`, `Circle3D`, `Line3D`, or customize.

### Usage
```rust
// Init parameter.
let parameter = SacAlgorithmParameter {
    probability: 0.99,
    threshold: 0.02,
    max_iterations: 2000,
    threads: 1,
};
// New and insert a Model of Plane.
let mut model = SacModelPlane::with_data(&vertices);
let mut algorithm = SacRansac {
    parameter,
    inliers: vec![],
};
// Compute and get result.
let result = algorithm.compute(&mut model);
if !result {
    println!("Segmentation Failed");
    return;
}
// Get Plane Coefficients.
let factor = model.get_coefficient();
// Get Points on plane.
let inlier = algorithm.inliers;
```

### Algorithm
Currently only support `RANSAC`.<br>
Algorithms use generic parameters.
```rust
#[derive(Debug, Clone, Copy)]
pub struct SacAlgorithmParameter {
    /// Probability: default `0.99`
    pub probability: f32,
    /// Value of threshold
    pub threshold: f32,
    /// If reach `max_iteration`, Optimization will be `terminate`.
    pub max_iterations: usize,
    /// Use parallel. Default `1` (single thread).
    pub threads: usize,
}
```
Algorithm implement below traits:
```rust
pub trait SacAlgorithmGetter {
    fn get_inliers(&self) -> &Vec<usize>;
}

pub trait SacAlgorithm<'a, P: Copy, T, R>: SacAlgorithmGetter
where
    T: BasicFloat,
    R: SacModel<'a, P, T>,
{
    fn compute(&mut self, model: &mut R) -> bool;
}
```

### Model
Represent model of shapes. Customize need to implement below.<br>

Model Trait:
```rust
pub trait SacModel<'a, P: Copy, T: BasicFloat> {
    type SampleIdxType;
    type CoefficientsType;

    const NB_SAMPLE: usize;
    const NB_COEFFICIENTS: usize;

    fn set_data(&mut self, data: &'a [P]);
    /// Set `NB_COEFFICIENTS` array.
    fn set_coefficient(&mut self, factor: &Self::CoefficientsType);
    /// Get `NB_COEFFICIENTS` array.
    fn get_coefficient(&self) -> Self::CoefficientsType;

    /// Get random sample points.
    fn samples(&self) -> &[P];
    /// Numbers of data
    fn data_len(&self) -> usize {
        self.samples().len()
    }
    /// Random numbers of indices by `NB_SAMPLE`.
    fn get_random_sample_id(&self) -> Vec<usize> {
        let mut rng = rand::thread_rng();
        let nb = self.data_len();
        use std::collections::HashSet;
        let mut set = HashSet::new();
        while set.len() < Self::NB_SAMPLE {
            set.insert(rng.gen_range(0..nb));
        }
        set.into_iter().collect()
    }
    /// Returns a distance list and uses `coefficients` to calculate the distance from data to the model.
    fn get_distance_to_model(&self, coefficients: &Self::CoefficientsType) -> Vec<T> {
        self.samples()
            .iter()
            .map(|&p| Self::compute_point_to_model(p, coefficients))
            .collect()
    }
    /// Return indices of distance between point and `coefficients` smaller than `tolerance`.
    fn select_indices_within_tolerance(
        &self,
        coefficients: &Self::CoefficientsType,
        tolerance: T,
    ) -> Vec<usize> {
        let data = self.samples();
        (0..data.len())
            .filter(|&i| Self::compute_point_to_model(data[i], coefficients) < tolerance)
            .collect()
    }
    /// Return numbers which between point and `coefficients` smaller than `tolerance`.
    fn count_indices_within_tolerance(
        &self,
        coefficients: &Self::CoefficientsType,
        tolerance: T,
    ) -> usize {
        let data = self.samples();
        (0..data.len())
            .filter(|&i| Self::compute_point_to_model(data[i], coefficients) < tolerance)
            .map(|_| 1)
            .sum()
    }
    /// Return distance between target `point` and `coefficients`.
    fn compute_point_to_model(p: P, coefficients: &Self::CoefficientsType) -> T;
    /// Get array of indices of samples.
    fn get_random_samples(&self) -> Self::SampleIdxType;
    /// Return `CoefficientsType` of samples.
    ///
    /// # Err
    /// * Numbers of data smaller than `NB_SAMPLE`.
    /// * Samples could not be computed.
    /// (ex: samples are overlay or parallel each other.)
    fn compute_model_coefficients(
        &self,
        samples: &Self::SampleIdxType,
    ) -> Result<Self::CoefficientsType, String>;
}
```

Models:
* SacModelPlane
* SacModelSphere
* SacModelCircle3d
* SacModelLine