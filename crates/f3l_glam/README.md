# F3l Glam
An improve crate to get and set for glam types.<br>
Because glam is `column-based`, it provides some traits to quickly obtain `row information`.

## F3lMatrix
Use to get row vector from matrix.
```rust
/// A trait of matrix for glam type
pub trait F3lMatrix {
    type RowType: Copy;

    // getter
    fn cols(&self) -> usize;
    fn rows(&self) -> usize;
    fn row(&self, r: usize) -> Self::RowType;
    /// get value with bound check
    fn get(&self, r: usize, c: usize) -> Option<f32>;
    fn at(&self, r: usize, c: usize) -> f32 {
        self.get(r, c).unwrap()
    }

    // setter
    fn set_row<R: Into<Self::RowType> + Copy>(&mut self, row: usize, v: R);
    /// pos: (row, col)
    fn set_element(&mut self, pos: (usize, usize), v: f32);
}
```
## ArrayRowMajor
To get row data and from / into row matrix.
```rust
/// A trait of from/to for glam types
pub trait ArrayRowMajor {
    type Row: Copy;
    type Mat: Copy;

    fn from_rows<R: Index<usize, Output = f32>>(rows: &[R]) -> Self;
    fn from_rows_slice(m: &[f32]) -> Self;
    fn to_rows_array(&self) -> Self::Row;
    fn from_cols_array_2d(m: &Self::Mat) -> Self;
    fn to_rows_array_2d(&self) -> Self::Mat;
    fn write_rows_to_slice(self, slice: &mut [f32]);
}

/// A dimension getter
pub trait ArrayDimensions {
    fn nb_cols() -> usize;
    fn nb_rows() -> usize;
}
```

## More Implementation
See each file.
```rust
pub mod mat2;
pub mod mat3;
pub mod mat3a;
pub mod mat4;
pub mod vec2;
pub mod vec3;
pub mod vec3a;
pub mod vec4;
```