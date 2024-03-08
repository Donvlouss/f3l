mod glam_extend;
// pub use glam_extend::*;

pub mod matrix3x3;

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
