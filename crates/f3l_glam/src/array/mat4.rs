impl super::F3lMatrix for glam::Mat4 {
    type RowType = glam::Vec4;

    #[inline]
    #[must_use]
    fn cols(&self) -> usize {
        4
    }

    #[inline]
    #[must_use]
    fn rows(&self) -> usize {
        4
    }

    #[inline]
    #[must_use]
    fn row(&self, r: usize) -> Self::RowType {
        glam::Vec4::new(
            self.x_axis[r],
            self.y_axis[r],
            self.z_axis[r],
            self.w_axis[r],
        )
    }

    #[inline]
    #[must_use]
    fn get(&self, r: usize, c: usize) -> Option<f32> {
        if r >= 4 {
            return None;
        }
        match c {
            0 => Some(self.x_axis[r]),
            1 => Some(self.y_axis[r]),
            2 => Some(self.z_axis[r]),
            3 => Some(self.w_axis[r]),
            _ => None,
        }
    }
    #[inline]
    #[must_use]
    fn set_row<R: Into<Self::RowType> + Copy>(&mut self, row: usize, v: R) {
        if row >= 4 {
            return;
        }
        self.x_axis[row] = v.into()[0];
        self.y_axis[row] = v.into()[1];
        self.z_axis[row] = v.into()[2];
        self.w_axis[row] = v.into()[3];
    }

    #[inline]
    #[must_use]
    fn set_element(&mut self, pos: (usize, usize), v: f32) {
        if pos.0 >= 4 || pos.1 >= 4 {
            return;
        }
        match pos.1 {
            0 => self.x_axis[pos.0] = v,
            1 => self.y_axis[pos.0] = v,
            2 => self.z_axis[pos.0] = v,
            3 => self.w_axis[pos.0] = v,
            _ => {}
        };
    }
}

impl super::ArrayRowMajor for glam::Mat4 {
    type Row = [f32; 16];

    type Mat = [[f32; 4]; 4];

    #[inline]
    #[must_use]
    fn from_rows<R: std::ops::Index<usize, Output = f32>>(rows: &[R]) -> Self {
        Self::from_cols_array(&[
            rows[0][0], rows[1][0], rows[2][0], rows[3][0], rows[0][1], rows[1][1], rows[2][1],
            rows[3][1], rows[0][2], rows[1][2], rows[2][2], rows[3][2], rows[0][3], rows[1][3],
            rows[2][3], rows[3][3],
        ])
    }

    #[inline]
    #[must_use]
    fn from_rows_slice(m: &[f32]) -> Self {
        Self::from_cols_array(&[
            m[0], m[4], m[8], m[12], m[1], m[5], m[9], m[13], m[2], m[6], m[10], m[14], m[3], m[7],
            m[11], m[15],
        ])
    }

    #[inline]
    #[must_use]
    fn to_rows_array(&self) -> Self::Row {
        self.transpose().to_cols_array()
    }

    #[inline]
    #[must_use]
    fn from_cols_array_2d(m: &Self::Mat) -> Self {
        Self::from_cols_array(&[
            m[0][0], m[1][0], m[2][0], m[3][0], m[0][1], m[1][1], m[2][1], m[3][1], m[0][2],
            m[1][2], m[2][2], m[3][2], m[0][3], m[1][3], m[2][3], m[3][3],
        ])
    }

    #[inline]
    #[must_use]
    fn to_rows_array_2d(&self) -> Self::Mat {
        self.transpose().to_cols_array_2d()
    }

    #[inline]
    #[must_use]
    fn write_rows_to_slice(self, slice: &mut [f32]) {
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

impl super::ArrayDimensions for glam::Mat4 {
    fn nb_cols() -> usize {
        4
    }

    fn nb_rows() -> usize {
        4
    }
}
impl super::GenericArray for glam::Mat4 {}
