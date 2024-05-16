# F3l Features
Data Features.

## Bounding
* AABB
* OBB
```rust
let obb = OBB::compute(&vertices);
// Get OBB 8 corners
let p0 = obb.center
    - obb.primary * obb.length[0]
    - obb.secondary * obb.length[1]
    - obb.tertiary * obb.length[2];
let p1 = p0 + obb.primary * obb.length[0] * 2.;
let p2 = p0 + obb.secondary * obb.length[1] * 2.;
let p3 = p0 + obb.tertiary * obb.length[2] * 2.;
let p4 = p2 + obb.primary * obb.length[0] * 2.;
let p5 = p1 + obb.tertiary * obb.length[2] * 2.;
let p6 = p2 + obb.tertiary * obb.length[2] * 2.;
let p7 = p4 + obb.tertiary * obb.length[2] * 2.;
```

# Normal Estimate
1. For each point search neighbors.
2. Compute eigenvector of neighbors.
3. The smallest eigenvalue one is which normal.

Normal Search Method [KDTree]
```rust
// Radius
let mut estimator = NormalEstimation::new(SearchBy::Radius(0.08f32));
// KNN
let mut estimator = NormalEstimation::new(SearchBy::Count(10));
```
```rust
// Compute!
if !estimator.compute(&vertices) {
    println!("Compute Normal Failed. Exit...");
    return;
}
let normals = estimator.normals();
```