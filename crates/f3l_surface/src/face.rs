pub enum FaceType<T> {
    Id(usize),
    Entry(T)
}

pub struct Face<T> {
    pub point: [FaceType<T>; 3]
}

#[derive(Debug, Clone, Copy)]
pub struct FaceIdType {
    pub point: [usize; 3]
}

impl<T> From<FaceIdType> for Face<T> {
    fn from(value: FaceIdType) -> Self {
        Self {
            point: [
                FaceType::Id(value.point[0]),
                FaceType::Id(value.point[1]),
                FaceType::Id(value.point[2]),
            ]
        }
    }
}
