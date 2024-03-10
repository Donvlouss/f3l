pub mod mat2;
pub mod mat3;
pub mod mat3a;
pub mod mat4;
pub mod vec2;
pub mod vec3;
pub mod vec3a;
pub mod vec4;

use std::ops::Index;


pub trait F3lMatrix {
    type RowType: Copy;

    // getter
    fn cols(&self) -> usize;
    fn rows(&self) -> usize;
    fn row(&self, r: usize) -> Self::RowType;
    fn get(&self, r: usize, c: usize) -> Option<f32>;
    fn at(&self, r: usize, c: usize) -> f32 {
        self.get(r, c).unwrap()
    }

    // setter
    fn set_row<R: Into<Self::RowType> + Copy>(&mut self, row: usize, v: R);
    /// pos: (row, col)
    fn set_element(&mut self, pos: (usize, usize), v: f32);
}

pub trait ArrayRowMajor
{
    type Row: Copy;
    type Mat: Copy;

    fn from_rows<R: Index<usize, Output=f32>>(rows: &[R]) -> Self;
    fn from_rows_slice(m: &[f32]) -> Self;
    fn to_rows_array(&self) -> Self::Row;
    fn from_cols_array_2d(m: &Self::Mat) -> Self;
    fn to_rows_array_2d(&self) -> Self::Mat;
    fn write_rows_to_slice(self, slice: &mut[f32]);
}