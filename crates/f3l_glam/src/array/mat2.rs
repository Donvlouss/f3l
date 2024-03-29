impl super::F3lMatrix for glam::Mat2 {
    type RowType = glam::Vec2;

    #[inline]
    fn cols(&self) -> usize {
        2
    }

    #[inline]
    fn rows(&self) -> usize {
        2
    }

    #[inline]
    fn row(&self, r: usize) -> Self::RowType {
        glam::Vec2::new(self.x_axis[r], self.y_axis[r])
    }

    #[inline]
    fn get(&self, r: usize, c: usize) -> Option<f32> {
        if r >= 2 {
            return None;
        }
        match c {
            0 => Some(self.x_axis[r]),
            1 => Some(self.y_axis[r]),
            _ => None,
        }
    }

    #[inline]
    fn set_row<R: Into<Self::RowType> + Copy>(&mut self, row: usize, v: R) {
        if row >= 2 {
            return;
        }
        self.x_axis[row] = v.into()[0];
        self.y_axis[row] = v.into()[1];
    }

    #[inline]
    fn set_element(&mut self, pos: (usize, usize), v: f32) {
        if pos.0 >= 2 || pos.1 >= 2 {
            return;
        }
        match pos.1 {
            0 => self.x_axis[pos.0] = v,
            1 => self.y_axis[pos.0] = v,
            _ => {}
        };
    }
}

impl super::ArrayRowMajor for glam::Mat2 {
    type Row = [f32; 4];

    type Mat = [[f32; 2]; 2];

    #[inline]
    fn from_rows<R: std::ops::Index<usize, Output = f32>>(rows: &[R]) -> Self {
        Self::from_cols_array(&[rows[0][0], rows[1][0], rows[0][1], rows[1][1]])
    }

    #[inline]
    fn from_rows_slice(m: &[f32]) -> Self {
        assert!(m.len() != 4);
        Self::from_cols_array(&[m[0], m[2], m[1], m[3]])
    }

    #[inline]
    fn to_rows_array(&self) -> Self::Row {
        self.transpose().to_cols_array()
    }

    #[inline]
    fn from_cols_array_2d(m: &Self::Mat) -> Self {
        Self::from_cols_array(&[m[0][0], m[1][0], m[0][1], m[1][1]])
    }

    #[inline]
    fn to_rows_array_2d(&self) -> Self::Mat {
        self.transpose().to_cols_array_2d()
    }

    #[inline]
    fn write_rows_to_slice(self, slice: &mut [f32]) {
        slice[0] = self.x_axis.x;
        slice[1] = self.y_axis.x;
        slice[2] = self.x_axis.y;
        slice[3] = self.y_axis.y;
    }
}

impl super::ArrayDimensions for glam::Mat2 {
    fn nb_cols() -> usize {
        2
    }

    fn nb_rows() -> usize {
        2
    }
}
impl super::GenericArray for glam::Mat2 {}
