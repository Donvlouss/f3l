impl super::F3lMatrix for glam::Mat3 {
    type RowType = glam::Vec3;

    #[inline]
    #[must_use]
    fn cols(&self) -> usize {
        3
    }
    
    #[inline]
    #[must_use]
    fn rows(&self) -> usize {
        3
    }

    #[inline]
    #[must_use]
    fn row(&self, r: usize) -> Self::RowType {
        glam::Vec3::new(self.x_axis[r], self.y_axis[r], self.z_axis[r])
    }

    #[inline]
    #[must_use]
    fn get(&self, r: usize, c: usize) -> Option<f32> {
        if r >= 3 {
            return None;
        }
        match c {
            0 => Some(self.x_axis[r]),
            1 => Some(self.y_axis[r]),
            2 => Some(self.z_axis[r]),
            _ => None
        }
    }

    #[inline]
    #[must_use]
    fn set_row<R: Into<Self::RowType> + Copy>(&mut self, row: usize, v: R) {
        if row >= 3 { return; }
        self.x_axis[row] = v.into()[0];
        self.y_axis[row] = v.into()[1];
        self.z_axis[row] = v.into()[2];
    }
    
    #[inline]
    #[must_use]
    fn set_element(&mut self, pos: (usize, usize), v: f32) {
        if pos.0 >= 3 || pos.1 >= 3 { return; }
        match pos.1 {
            0 => self.x_axis[pos.0] = v,
            1 => self.y_axis[pos.0] = v,
            2 => self.z_axis[pos.0] = v,
            _ => {}
        };
    }
}

impl super::ArrayRowMajor for glam::Mat3
{
    type Row = [f32; 9];

    type Mat = [[f32; 3]; 3];

    #[inline]
    #[must_use]
    fn from_rows<R: std::ops::Index<usize, Output=f32>>(rows: &[R]) -> Self
    {
        Self::from_cols_array(&[rows[0][0], rows[1][0], rows[2][0], rows[0][1], rows[1][1], rows[2][1]
            , rows[0][2], rows[1][2], rows[2][2]])
    }

    #[inline]
    #[must_use]
    fn from_rows_slice(m: &[f32]) -> Self {
        Self::from_cols_array(&[m[0], m[3], m[6], m[1], m[4], m[7], m[2], m[5], m[8]])
    }

    #[inline]
    #[must_use]
    fn to_rows_array(&self) -> Self::Row {
        self.transpose().to_cols_array()
    }

    #[inline]
    #[must_use]
    fn from_cols_array_2d(m: &Self::Mat) -> Self {
        Self::from_cols_array(&[m[0][0], m[1][0], m[2][0], m[0][1], m[1][1], m[2][1], m[0][2], m[1][2], m[2][2]])
    }

    #[inline]
    #[must_use]
    fn to_rows_array_2d(&self) -> Self::Mat {
        self.transpose().to_cols_array_2d()
    }

    #[inline]
    #[must_use]
    fn write_rows_to_slice(self, slice: &mut[f32]) {
        slice[0] = self.x_axis.x;
        slice[1] = self.x_axis.y;
        slice[2] = self.x_axis.z;
        slice[3] = self.y_axis.x;
        slice[4] = self.y_axis.y;
        slice[5] = self.y_axis.z;
        slice[6] = self.z_axis.x;
        slice[7] = self.z_axis.y;
        slice[8] = self.z_axis.z;
    }
}
