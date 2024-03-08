use glam::{
    Mat2, Mat3, Mat3A, Mat4,
    Vec2, Vec3, Vec3A, Vec4
};

use crate::F3lMatrix;

impl super::F3lMatrix for Mat2 {
    type RowType = glam::Vec2;
    
    fn cols(&self) -> usize {
        2
    }

    fn rows(&self) -> usize {
        2
    }

    fn row(&self, r: usize) -> Self::RowType {
        glam::Vec2::new(self.x_axis[r], self.y_axis[r])
    }

    fn get(&self, r: usize, c: usize) -> Option<f32> {
        if r >= 2 {
            return None;
        }
        match c {
            0 => Some(self.x_axis[r]),
            1 => Some(self.y_axis[r]),
            _ => None
        }
    }
    
    fn set_row<R: Into<Self::RowType> + Copy>(&mut self, row: usize, v: R) {
        if row >= 2 { return; }
        self.x_axis[row] = v.into()[0];
        self.y_axis[row] = v.into()[1];
    }
    
    fn set_element(&mut self, pos: (usize, usize), v: f32) {
        if pos.0 >= 2 || pos.1 >= 2 { return; }
        match pos.1 {
            0 => self.x_axis[pos.0] = v,
            1 => self.y_axis[pos.0] = v,
            _ => {}
        };
    }
    
}

impl super::F3lMatrix for Mat3 {
    type RowType = glam::Vec3;

    fn cols(&self) -> usize {
        3
    }
    
    fn rows(&self) -> usize {
        3
    }

    fn row(&self, r: usize) -> Self::RowType {
        glam::Vec3::new(self.x_axis[r], self.y_axis[r], self.z_axis[r])
    }

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

    fn set_row<R: Into<Self::RowType> + Copy>(&mut self, row: usize, v: R) {
        if row >= 3 { return; }
        self.x_axis[row] = v.into()[0];
        self.y_axis[row] = v.into()[1];
        self.z_axis[row] = v.into()[2];
    }
    
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

impl super::F3lMatrix for Mat3A {
    type RowType = glam::Vec3A;

    fn cols(&self) -> usize {
        3
    }

    fn rows(&self) -> usize {
        3
    }

    fn row(&self, r: usize) -> Self::RowType {
        glam::Vec3A::new(self.x_axis[r], self.y_axis[r], self.z_axis[r])
    }

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

    fn set_row<R: Into<Self::RowType> + Copy>(&mut self, row: usize, v: R) {
        if row >= 3 { return; }
        self.x_axis[row] = v.into()[0];
        self.y_axis[row] = v.into()[1];
        self.z_axis[row] = v.into()[2];
    }
    
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

impl super::F3lMatrix for Mat4 {
    type RowType = glam::Vec4;

    fn cols(&self) -> usize {
        4
    }

    fn rows(&self) -> usize {
        4
    }

    fn row(&self, r: usize) -> Self::RowType {
        glam::Vec4::new(self.x_axis[r], self.y_axis[r], self.z_axis[r], self.w_axis[r])
    }

    fn get(&self, r: usize, c: usize) -> Option<f32> {
        if r >= 4 {
            return None;
        }
        match c {
            0 => Some(self.x_axis[r]),
            1 => Some(self.y_axis[r]),
            2 => Some(self.z_axis[r]),
            3 => Some(self.w_axis[r]),
            _ => None
        }
    }
    fn set_row<R: Into<Self::RowType> + Copy>(&mut self, row: usize, v: R) {
        if row >= 4 { return; }
        self.x_axis[row] = v.into()[0];
        self.y_axis[row] = v.into()[1];
        self.z_axis[row] = v.into()[2];
        self.w_axis[row] = v.into()[3];
    }
    
    fn set_element(&mut self, pos: (usize, usize), v: f32) {
        if pos.0 >= 4 || pos.1 >= 4 { return; }
        match pos.1 {
            0 => self.x_axis[pos.0] = v,
            1 => self.y_axis[pos.0] = v,
            2 => self.z_axis[pos.0] = v,
            3 => self.w_axis[pos.0] = v,
            _ => {}
        };
    }
}

impl F3lMatrix for Vec2 {
    type RowType = [f32; 1];

    fn cols(&self) -> usize {
        1
    }

    fn rows(&self) -> usize {
        2
    }

    fn row(&self, r: usize) -> Self::RowType {
        [self[r]]
    }

    fn get(&self, r: usize, _: usize) -> Option<f32> {
        match r {
            0 => Some(self.x),
            1 => Some(self.y),
            _ => None
        }
    }
    
    fn set_row<R: Into<Self::RowType> + Copy>(&mut self, row: usize, v: R) {
        if row >=1 { return; }
        self[row] = v.into()[0];
    }
    
    fn set_element(&mut self, pos: (usize, usize), v: f32) {
        if pos.1 >=1 { return; }
        self[pos.0] = v;
    }
}

impl F3lMatrix for Vec3 {
    type RowType = [f32; 1];

    fn cols(&self) -> usize {
        1
    }

    fn rows(&self) -> usize {
        3
    }

    fn row(&self, r: usize) -> Self::RowType {
        [self[r]]
    }

    fn get(&self, r: usize, _: usize) -> Option<f32> {
        match r {
            0 => Some(self.x),
            1 => Some(self.y),
            2 => Some(self.z),
            _ => None
        }
    }
    
    fn set_row<R: Into<Self::RowType> + Copy>(&mut self, row: usize, v: R) {
        if row >=1 { return; }
        self[row] = v.into()[0];
    }
    
    fn set_element(&mut self, pos: (usize, usize), v: f32) {
        if pos.1 >=1 { return; }
        self[pos.0] = v;
    }
}

impl F3lMatrix for Vec3A {
    type RowType = [f32; 1];

    fn cols(&self) -> usize {
        1
    }

    fn rows(&self) -> usize {
        3
    }

    fn row(&self, r: usize) -> Self::RowType {
        [self[r]]
    }

    fn get(&self, r: usize, _: usize) -> Option<f32> {
        match r {
            0 => Some(self.x),
            1 => Some(self.y),
            2 => Some(self.z),
            _ => None
        }
    }
    
    fn set_row<R: Into<Self::RowType> + Copy>(&mut self, row: usize, v: R) {
        if row >=1 { return; }
        self[row] = v.into()[0];
    }
    
    fn set_element(&mut self, pos: (usize, usize), v: f32) {
        if pos.1 >=1 { return; }
        self[pos.0] = v;
    }
}

impl F3lMatrix for Vec4 {
    type RowType = [f32; 1];

    fn cols(&self) -> usize {
        1
    }

    fn rows(&self) -> usize {
        4
    }

    fn row(&self, r: usize) -> Self::RowType {
        [self[r]]
    }

    fn get(&self, r: usize, _: usize) -> Option<f32> {
        match r {
            0 => Some(self.x),
            1 => Some(self.y),
            2 => Some(self.z),
            3 => Some(self.w),
            _ => None
        }
    }
    
    fn set_row<R: Into<Self::RowType> + Copy>(&mut self, row: usize, v: R) {
        if row >=1 { return; }
        self[row] = v.into()[0];
    }
    
    fn set_element(&mut self, pos: (usize, usize), v: f32) {
        if pos.1 >=1 { return; }
        self[pos.0] = v;
    }
}