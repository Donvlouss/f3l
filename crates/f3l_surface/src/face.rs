/// Face Types.
pub enum FaceType<T> {
    /// Vertex Id of data.
    Id(usize),
    /// Vertex Entry of data.
    Entry(T)
}

/// Generic Face object.
pub struct Face<T> {
    pub point: [FaceType<T>; 3]
}

/// Face Instance object.
#[derive(Debug, Clone, Copy)]
pub struct FaceInstanceType<P: Copy> {
    pub point: [P; 3],
}

/// Face Id object.
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

impl<P: Copy> From<FaceInstanceType<P>> for Face<P> {
    fn from(value: FaceInstanceType<P>) -> Self {
        Self {
            point: [
                FaceType::Entry(value.point[0]),
                FaceType::Entry(value.point[1]),
                FaceType::Entry(value.point[2]),
            ]
        }
    }
}