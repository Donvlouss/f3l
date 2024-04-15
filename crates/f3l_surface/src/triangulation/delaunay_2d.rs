use std::collections::{HashMap, HashSet};

use f3l_core::{find_circle, get_minmax, EdgeLinker};

use crate::Delaunay2DShape;

use super::{BasicFloat, FaceIdType, SubTriangle};

/// Compute Delaunay Triangles with alpha shape. Implement [Bowyer–Watson algorithm](https://en.wikipedia.org/wiki/Bowyer%E2%80%93Watson_algorithm)
/// 
/// 1. Generate a super-triangle which could contains all data.
/// 2. Recursive insert point, and generate sub-triangles.
/// 3. Remove triangles which linked to super-triangles.
/// 4. Remove triangle which radius of circumscribed smaller than `alpha`.
/// 5. Find all contours (non-common edge).
/// 6. Classify contours to each shapes.
#[derive(Debug, Clone)]
pub struct Delaunay2D<'a, T: BasicFloat, P> {
    pub data: &'a [P],
    pub super_triangle: [[T;2];3],
    pub shapes: Vec<Delaunay2DShape>,
}

impl<'a, T: BasicFloat, P> Delaunay2D<'a, T, P>
where
    P: Into<[T; 2]> + Copy + std::ops::Index<usize, Output = T>,
    [T; 2]: Into<P>,
{
    pub fn new(data: &'a [P]) -> Self {
        Self {
            data, 
            super_triangle: [[T::zero(); 2]; 3],
            shapes: vec![]
        }
    }

    /// Return point data at `id`.
    /// 
    /// Cause data is a reference, could not add vertices of super-triangles 
    /// to data. Using this method to check `id` if over than data, if true,
    /// which may get vertex of super-triangle.
    /// 
    /// # Panic
    /// `id` is larger than `nb of data` + 3 (vertices of super-triangle).
    #[inline]
    pub fn at(&self, id: usize) -> [T; 2] {
        match id >= self.data.len() {
            true => self.super_triangle[id - self.data.len()],
            false => self.data[id].into(),
        }
    }

    /// Return center and radius of circumscribed.
    /// 
    /// Compute triangles circumscribed circle.
    pub fn find_out_circle(&self, ids: [usize; 3]) -> ([T; 2], T) {
        let p1 = self.at(ids[0]);
        let p2 = self.at(ids[1]);
        let p3 = self.at(ids[2]);
        let p1 = [p1[0], p1[1], T::zero()];
        let p2 = [p2[0], p2[1], T::zero()];
        let p3 = [p3[0], p3[1], T::zero()];
        
        let (pc, _, radius) = find_circle(&[p1, p2, p3]);

        ([pc[0], pc[1]], radius*radius)

    }

    /// Return [`SubTriangle`] instance.
    /// 
    /// A factory of creating a [`SubTriangle`] by three ids.
    pub fn generate_triangle(&self, ids: [usize; 3]) -> SubTriangle<T> {
        let (center, radius) = self.find_out_circle(ids);
        SubTriangle {
            tri: FaceIdType { point: ids },
            removed: false,
            center, radius
        }
    }

    /// Return super-triangle.
    pub fn find_super_triangle(&mut self) -> SubTriangle<T> {
        let minmax = get_minmax(self.data);
        let c = [
            (minmax.0[0] + minmax.1[0]) / T::from(2.).unwrap(),
            (minmax.0[1] + minmax.1[1]) / T::from(2.).unwrap(),
        ];
        let dim = [
            (minmax.0[0] - minmax.1[0]).abs() / T::from(2.).unwrap(),
            (minmax.0[1] - minmax.1[1]).abs() / T::from(2.).unwrap(),
        ];
        let top = [c[0],  c[1] + T::from(3.).unwrap() * dim[1]];
        let bottom_line = c[1] - T::from(3.).unwrap() * dim[1];
        let m = dim[1] / dim[0];
        let b1 = top[1] - m * top[0];
        let b2 = top[1] + m * top[0];
        let x1 = (bottom_line - b1) /  m;
        let x2 = (bottom_line - b2) / -m;
        
        self.super_triangle = [
            top,
            [x1, bottom_line],
            [x2, bottom_line]
        ];
        let n = self.data.len();
        self.generate_triangle([n, n+1, n+2])
    }

    /// Insert point to current delaunay triangles.
    /// 
    /// 1. Find triangle which point is inside the circumscribed circle.
    /// 2. Mark found triangles to removed, and save edges. 
    /// 3. Find non-common edges as a hole, and found the ring of hole.
    /// 4. Link point to edges of ring as new triangles, then push to queue.
    pub fn insert(&mut self, id: usize, triangles: &mut Vec<SubTriangle<T>>) {
        let mut new_faces = vec![];
        let mut edges = HashMap::new();
        let p = self.at(id);
        (0..triangles.len()).for_each(|i| {
            let SubTriangle { tri, removed, center, radius } = &mut triangles[i];
            let d = (center[0] - p[0]).powi(2) + (center[1] - p[1]).powi(2);
            if d <= *radius {
                *removed = true;
                let pts = tri.point;
                [(pts[0], pts[1]), (pts[0], pts[2]), (pts[1], pts[2])].into_iter().for_each(|(e0, e1)| {
                    let e = if e0 < e1 { (e0, e1) } else { (e1, e0) };
                    let entry = edges.entry(e).or_insert(0);
                    *entry += 1;
                });
            }
        });
        edges.into_iter().filter(|&(_, v)| v <= 1)
            .for_each(|(k, _)| {
                let face = self.generate_triangle([id, k.0, k.1]);
                triangles.push(face);
                new_faces.push(face);
            });
    }

    /// Compute delaunay triangles, and using alpha to remove invalid triangles.
    /// 
    /// 1. Find super-triangle, push to queue.
    /// 2. Insert each point. see [`Delaunay2D::insert`]
    /// 3. Split triangle and contours by alpha.
    /// 4. Found contours of each shapes.
    pub fn compute(&mut self, alpha: T) {
        let mut triangles = vec![self.find_super_triangle()];
        let n = self.data.len();
        for i in 0..n {
            self.insert(i, &mut triangles);
            triangles = triangles.into_iter().filter(|tri| !tri.removed).collect();
        };
        // Remove triangles connected to super-triangle.
        let out_ids = [n, n+1, n+2];
        triangles = triangles.into_iter().filter_map(|tri| {
            let SubTriangle { tri: face, removed, ..} = tri;
            if removed { return None; }
            if face.point.iter().any(|i| out_ids.contains(i)) {
                return None;
            }
            Some(tri)
        }).collect();
        
        let (mesh, contours) = self.compute_alpha(&triangles, alpha*alpha);
        
        let mut solver = EdgeLinker::new(&contours);
        solver.search(false);

        self.shapes = Self::cluster_alpha_shapes(mesh, solver.closed);
    }

    /// Compute alpha shape
    /// 
    /// 1. Remove triangles which radius of circumscribed larger than alpha.
    /// 2. Split to meshes an contours.
    pub fn compute_alpha(&self, triangles: &Vec<SubTriangle<T>>, alpha: T) -> (Vec<SubTriangle<T>>, Vec<(usize, usize)>) {
        let mut inner = HashMap::new();
        let inner_triangles = triangles.iter().filter_map(|&face| {
            if face.radius < alpha {
                [(0_usize, 1), (0, 2), (1, 2)].into_iter().for_each(|e| {
                    let (e0, e1) = (face.tri.point[e.0], face.tri.point[e.1]);
                    let edge = if e0 < e1 {(e0, e1)} else {(e1, e0)};
                    let entry = inner.entry(edge).or_insert(0);
                    *entry += 1;
                });
                Some(face)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

        (
            inner_triangles,
            inner.into_iter().filter_map(|(k, v)| {
                if v <= 1 {
                    Some(k)
                } else {
                    None
                }
            }).collect()
        )
    }

    /// Return list of [`Delaunay2DShape`].
    /// 
    /// 1. Search cluster of triangles which have common-edge.
    /// 2. Search owned contours of each shape.
    pub fn cluster_alpha_shapes(triangles: Vec<SubTriangle<T>>, contours: Vec<Vec<(usize, usize)>>) -> Vec<Delaunay2DShape> {
        // If there is only one contour, means all triangles in one shape, and has no holes.
        if contours.len() == 1 {
            return vec![
                Delaunay2DShape { 
                    mesh: triangles.into_iter().map(|tri| tri.tri).collect(),
                    contours
                }
            ];
        }


        let mut triangles = triangles;
        for tri in triangles.iter_mut() {
            tri.removed = false;
        }

        let mut shapes = vec![];
        (0..triangles.len()).for_each(|i| {
            if triangles[i].removed {
                return;
            }
            
            let mut selected = vec![triangles[i].tri];
            let mut edges = HashSet::new();
            {
                let tri = &mut triangles[i];
                tri.removed = true;
            }
            
            let ps = triangles[i].tri.point;
            [(0, 1), (1, 2), (2, 0)].into_iter().for_each(|(a, b)| {
                edges.insert(if ps[a] < ps[b] {(ps[a], ps[b])} else {(ps[b], ps[a])});
            });

            let mut n = selected.len();
            loop {
                (i+1..triangles.len()).for_each(|ii| {
                    if triangles[ii].removed {
                        return;
                    }

                    let ps = triangles[ii].tri.point;
                    let es = [(0, 1), (1, 2), (2, 0)].into_iter().map(|(a, b)|
                        if ps[a] < ps[b] {(ps[a], ps[b])} else {(ps[b], ps[a])}).collect::<Vec<_>>();
                    if es.iter().any(|e| edges.contains(e)) {
                        es.iter().for_each(|e| {edges.insert(*e);});
                        selected.push(triangles[ii].tri);

                        let tri = &mut triangles[ii];
                        tri.removed = true;
                    }
                });
                if n == selected.len() {
                    break;
                }
                n = selected.len();
            }

            let mut shape_contours = vec![];
            for ii in 0..contours.len() {
                if contours[ii].iter().all(|&(a, b)| {
                    let e = if a < b {(a, b)} else {(b, a)};
                    edges.contains(&e)
                }) {
                    shape_contours.push(contours[ii].clone());
                }
            };

            shapes.push(Delaunay2DShape{
                mesh: selected,
                contours: shape_contours,
            })
        });

        shapes
    }
}
