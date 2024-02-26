#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum SearchBy
{
    Count(usize),
    Radius(f32)
}

pub trait TreeSearch<P>
{
    fn search_knn(&self, point: &P, k: usize) -> Vec<(P, f32)>;
    fn search_radius(&self, point: &P, radius: f32) -> Vec<P>;
}

pub trait TreeResult
{
    type T;
    type Output;
    fn new(arg: Self::T) -> Self;
    fn with_capacity(arg: Self::T, capacity: usize) -> Self;
    fn result(&self) -> Vec<Self::Output>;

    fn add(&mut self, data: usize, distance: f32);
    fn is_full(&self) -> bool;
    fn worst(&self) -> f32;
    fn clear(&mut self);
}

pub struct TreeKnnResult
{
    pub data: Vec<(usize, f32)>,
    pub size: usize,
    pub count: usize,
    pub farthest: f32
}

impl TreeResult for TreeKnnResult
{
    type T = usize;
    type Output = (usize, f32);

    fn new(arg: Self::T) -> Self {
        Self {
            data: Vec::with_capacity(arg),
            size: arg,
            count: 0,
            farthest: f32::MAX
        }
    }

    fn with_capacity(arg: Self::T, capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(arg),
            size: capacity,
            count: 0,
            farthest: f32::MAX
        }
    }

    fn result(&self) -> Vec<Self::Output> {
        let mut queue = self.data.clone();
        queue.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        queue
    }

    fn add(&mut self, data: usize, distance: f32)
    {
        let mut need_sort = false;
        if self.count < self.size
        {
            need_sort = true;
            self.data.push((data, distance));
            self.count += 1;
        } else {
            if distance > self.farthest {
                return;
            }
            let idx = self.data.partition_point(|x| x.1 < distance);
            self.data.insert(idx, (data, distance));
            self.data.pop();
        }
        if self.count == self.size {
            // Only sort when data is full
            if need_sort {
                self.data.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            }
            self.farthest = self.data.last().unwrap().1;
        }
    }

    fn is_full(&self) -> bool
    {
        self.count >= self.size
    }

    fn worst(&self) -> f32 {
        self.farthest
    }

    fn clear(&mut self) {
        self.data.clear();
        self.count = 0;
    }

}

pub struct TreeRadiusResult
{
    pub data: Vec<usize>,
    pub count: usize,
    pub radius: f32,
    pub size: Option<usize>,
}

impl TreeRadiusResult {
    pub fn set_to_maximum_size(self, maximum_size: usize) -> Self {
        let TreeRadiusResult { data, count, radius, ..} = self;
        Self {
            data,
            count,
            radius,
            size: Some(maximum_size)
        }
    }

    pub fn set_maximum_size(&mut self, maximum_size: usize) {
        self.size = Some(maximum_size);
    }

    pub fn maximum_size(&self) -> Option<usize> {
        self.size
    }
}

impl TreeResult for TreeRadiusResult
{
    type T = f32;
    type Output = usize;

    fn new(arg: Self::T) -> Self {
        Self {
            data: Vec::new(),
            count: 0,
            radius: arg,
            size: None
        }
    }

    fn with_capacity(arg: Self::T, capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            count: 0,
            radius: arg,
            size: None,
        }
    }

    fn result(&self) -> Vec<Self::Output> {
        self.data.clone()
    }

    fn add(&mut self, data: usize, distance: f32) {
        if distance > self.radius {
            return;
        }
        self.data.push(data);
        self.count += 1;
    }

    fn is_full(&self) -> bool {
        match self.size {
            None => false,
            Some(size) => self.count >= size,
        }
    }

    fn worst(&self) -> f32 {
        self.radius
    }

    fn clear(&mut self) {
        self.data.clear();
    }
}