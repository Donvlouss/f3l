impl super::F3lMatrix for glam::Vec4 {
    type RowType = [f32; 1];

    #[inline]
    fn cols(&self) -> usize {
        1
    }

    #[inline]
    fn rows(&self) -> usize {
        4
    }

    #[inline]
    fn row(&self, r: usize) -> Self::RowType {
        [self[r]]
    }

    #[inline]
    fn get(&self, r: usize, _: usize) -> Option<f32> {
        match r {
            0 => Some(self.x),
            1 => Some(self.y),
            2 => Some(self.z),
            3 => Some(self.w),
            _ => None,
        }
    }

    #[inline]
    fn set_row<R: Into<Self::RowType> + Copy>(&mut self, row: usize, v: R) {
        if row >= 1 {
            return;
        }
        self[row] = v.into()[0];
    }

    #[inline]
    fn set_element(&mut self, pos: (usize, usize), v: f32) {
        if pos.1 >= 1 {
            return;
        }
        self[pos.0] = v;
    }
}

impl super::ArrayRowMajor for glam::Vec4 {
    type Row = [f32; 4];

    type Mat = [[f32; 1]; 4];

    #[inline]
    fn from_rows<R: std::ops::Index<usize, Output = f32>>(rows: &[R]) -> Self {
        Self::new(rows[0][0], rows[0][1], rows[0][3], rows[0][3])
    }

    #[inline]
    fn from_rows_slice(m: &[f32]) -> Self {
        Self::from_slice(m)
    }

    #[inline]
    fn to_rows_array(&self) -> Self::Row {
        (*self).into()
    }

    #[inline]
    fn from_cols_array_2d(m: &Self::Mat) -> Self {
        Self::new(m[0][0], m[1][0], m[2][0], m[3][0])
    }

    #[inline]
    fn to_rows_array_2d(&self) -> Self::Mat {
        [[self.x], [self.y], [self.z], [self.w]]
    }

    #[inline]
    fn write_rows_to_slice(self, slice: &mut [f32]) {
        self.write_to_slice(slice)
    }
}

impl super::ArrayDimensions for glam::Vec4 {
    fn nb_cols() -> usize {
        4
    }

    fn nb_rows() -> usize {
        1
    }
}
impl super::GenericArray for glam::Vec4 {}
