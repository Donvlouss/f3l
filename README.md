# F3L (Fusion 3D Library)
[<img alt="crates.io" src="https://img.shields.io/crates/v/f3l.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/f3l)

To become a 3d library in rust.

Some features could also use on 2d.

See more for each `README.md` of crates.

`serde` support.

| crate | description |
|-------|-------------|
|f3l_glam| Some trait and implement for glam types.|
|f3l_core| General mathematics and types definition.|
|f3l_search_tree| Search Tree for finding neighbors. (n-dim)|
|f3l_filter| 3D and 2D points filter.|
|f3l_segmentation| 3D and 2D Cluster and Model Optimize.|
|f3l_features| 3D and 2D data features.|
|f3l_surface| Compute Hulls and triangulation.|

## Data of examples
|file|source|
|----|------|
|table_scene_lms400.ply| pcd to ply. [Generate from pcl.](https://github.com/PointCloudLibrary/data/blob/master/tutorials/table_scene_lms400.pcd)|
|table_voxel_down.ply| Using voxel down to generate table_scene_lms400.ply|
|table_remove_plane.ply| Remove planes of ground and table|
[Data](https://github.com/Donvlouss/f3l_data)
