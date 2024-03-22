pub mod mat2;
pub mod mat3;
pub mod mat3a;
pub mod mat4;
pub mod vec2;
pub mod vec3;
pub mod vec3a;
pub mod vec4;

use std::ops::Index;

use num_traits::NumCast;

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

pub trait ArrayDimensions {
    fn nb_cols() -> usize;
    fn nb_rows() -> usize;
}

pub trait GenericArray: ArrayDimensions + Sized {
    fn cast_from<T: NumCast, const C: usize, const R: usize>(from: [[T; C]; R]) -> Self
    where
        Self: ArrayRowMajor<Mat = [[f32; R]; C]>,
    {
        // type ArrayRowMajor::Mat = [[f32; R]; C];
        let mut cast = [[0f32; R]; C];
        (0..Self::nb_rows()).for_each(|r| {
            let row_set_0 = R <= r;
            (0..Self::nb_cols()).for_each(|c| {
                let col_set_0 = C <= c;
                if row_set_0 || col_set_0 {
                    cast[c][r] = 0f32;
                }
                cast[c][r] = from[c][r].to_f32().unwrap();
            });
        });
        // Self::from_cols_array_2d(&cast)
        Self::from_cols_array_2d(&cast)
    }
}
