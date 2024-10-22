# F3l Search Tree
Search Tree implementation.
Input data is need to implement `Into<[T; D]>`

## Trees: 
* KD-Tree
* OC-Tree

## Tree Search Parameter
Search Method
* Count : KNN
* Radius: Radius Search
```rust
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum SearchBy {
    Count(usize),
    Radius(f32),
}
```

## Data
* Array
```rust
pub fn test_insert_data_array() {
    let mut tree = KdTree::<f32, 3>::new();
    tree.set_data(&vec![[1f32, 2f32, 3f32], [4f32, 5f32, 6f32]]);

    let mut d = 1.0f32;
    tree.data.iter().for_each(|element| {
        element.iter().for_each(|e| {
            assert_relative_eq!(d, e);
            d += 1f32;
        })
    });
}
```
* Glam:
```rust
pub fn test_insert_data_glam() {
    let mut tree = KdTree::<f32, 3>::new();
    tree.set_data(&vec![Vec3::new(1.0, 2.0, 3.0), Vec3::new(4.0, 5.0, 6.0)]);

    let mut d = 1.0f32;
    tree.data.iter().for_each(|element| {
        element.iter().for_each(|e| {
            assert_relative_eq!(d, e);
            d += 1f32;
        })
    });
}
```
* Nalgebra:
```rust
pub fn test_insert_data_nalgebra() {
    let mut tree = KdTree::<f32, 3>::new();
    tree.set_data(&vec![
        Point3::<f32>::new(1.0, 2.0, 3.0),
        Point3::<f32>::new(4.0, 5.0, 6.0),
    ]);

    let mut d = 1.0f32;
    tree.data.iter().for_each(|element| {
        element.iter().for_each(|e| {
            assert_relative_eq!(d, e);
            d += 1f32;
        })
    });
}
```
* Custom
```rust
#[derive(Debug, Clone, Copy)]
struct MyStruct {
    x: f32,
    y: f32,
    z: f32,
}

impl From<[f32; 3]> for MyStruct {
    fn from(value: [f32; 3]) -> Self {
        MyStruct {
            x: value[0],
            y: value[1],
            z: value[2],
        }
    }
}

impl From<MyStruct> for [f32; 3] {
    fn from(value: MyStruct) -> Self {
        [value.x, value.y, value.z]
    }
}

#[test]
pub fn test_insert_data_custom() {
    let mut tree = KdTree::<f32, MyStruct>::new();
    tree.set_data(&vec![
        MyStruct {
            x: 1.0f32,
            y: 2.0f32,
            z: 3.0f32,
        },
        MyStruct {
            x: 4.0f32,
            y: 5.0f32,
            z: 6.0f32,
        },
    ]);
    let mut d = 1.0f32;
    tree.data.iter().for_each(|element| {
        element.iter().for_each(|e| {
            assert_relative_eq!(d, e);
            d += 1f32;
        })
    });
}
```


## Search
Return indices of data or instance.
```rust
pub trait TreeSearch<P> {
    fn search_knn_ids(&self, point: &P, k: usize) -> Vec<usize>;
    fn search_radius_ids(&self, point: &P, radius: f32) -> Vec<usize>;

    fn search_knn(&self, point: &P, k: usize) -> Vec<(P, f32)>;
    fn search_radius(&self, point: &P, radius: f32) -> Vec<P>;
}
```

## Result
* KNN
```rust
#[derive(Debug, Clone)]
pub struct TreeKnnResult {
    /// KNN ids and distances.
    pub data: Vec<(usize, f32)>,
    /// Target of `K`.
    pub size: usize,
    /// Length of data.
    pub count: usize,
    /// Used in searching.
    pub farthest: f32,
}
```
* Radius
```rust
#[derive(Debug, Clone)]
pub struct TreeRadiusResult {
    /// Neighbors in radius.
    pub data: Vec<usize>,
    /// Length of data
    pub count: usize,
    /// Target radius
    pub radius: f32,
    /// `Optional`: full check when `count` more than `size`
    pub size: Option<usize>,
}
```

## Ignore
Add Ignore to `Search Trees`. Could use to sort from first target.

```rust
let first = // first element.
let mut sorted = vec![first];
let mut tree = KdTree::new(3);
tree.set_data(cloud);
tree.build();
tree.set_ignore(true);
tree.add_ignore(first);

while sorted.len() < cloud.len() {
    let src = sorted.last().unwrap();
    let pts = tree.search_knn_ids(&cloud[*src], 1);
    assert!(!pts.is_empty(), "Should at least 1 point.");
    let p = pts[0];
    sorted.push(p);
    tree.add_ignore(p);
}

```