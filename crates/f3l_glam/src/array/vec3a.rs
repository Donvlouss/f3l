
impl super::F3lMatrix for glam::Vec3A {
    type RowType = [f32; 1];

    #[inline]
    #[must_use]
    fn cols(&self) -> usize {
        1
    }

    #[inline]
    #[must_use]
    fn rows(&self) -> usize {
        3
    }

    #[inline]
    #[must_use]
    fn row(&self, r: usize) -> Self::RowType {
        [self[r]]
    }

    #[inline]
    #[must_use]
    fn get(&self, r: usize, _: usize) -> Option<f32> {
        match r {
            0 => Some(self.x),
            1 => Some(self.y),
            2 => Some(self.z),
            _ => None
        }
    }
    
    #[inline]
    #[must_use]
    fn set_row<R: Into<Self::RowType> + Copy>(&mut self, row: usize, v: R) {
        if row >=1 { return; }
        self[row] = v.into()[0];
    }
    
    #[inline]
    #[must_use]
    fn set_element(&mut self, pos: (usize, usize), v: f32) {
        if pos.1 >=1 { return; }
        self[pos.0] = v;
    }
}

impl super::ArrayRowMajor for glam::Vec3A {
    type Row = [f32; 3];

    type Mat = [[f32; 1]; 3];

    #[inline]
    #[must_use]
    fn from_rows<R: std::ops::Index<usize, Output=f32>>(rows: &[R]) -> Self {
        Self::new(rows[0][0], rows[0][1], rows[0][3])
    }

    #[inline]
    #[must_use]
    fn from_rows_slice(m: &[f32]) -> Self {
        Self::from_slice(m)
    }

    #[inline]
    #[must_use]
    fn to_rows_array(&self) -> Self::Row {
        (*self).into()
    }

    #[inline]
    #[must_use]
    fn from_cols_array_2d(m: &Self::Mat) -> Self {
        Self::new(m[0][0], m[0][1], m[0][2])
    }

    #[inline]
    #[must_use]
    fn to_rows_array_2d(&self) -> Self::Mat {
        [[self.x], [self.y], [self.z]]
    }

    #[inline]
    #[must_use]
    fn write_rows_to_slice(self, slice: &mut[f32]) {
        self.write_to_slice(slice)
    }
}

impl super::ArrayDimensions for glam::Vec3A {
    fn nb_cols() -> usize {
        3
    }

    fn nb_rows() -> usize {
        1
    }
}
impl super::GenericArray for glam::Vec3A {}
