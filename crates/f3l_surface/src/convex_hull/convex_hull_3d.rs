use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    ops::Index,
};

use f3l_core::{
    apply_both,
    serde::{self, Deserialize, Serialize},
    BasicFloat, Line, SimpleSliceMath,
};

use crate::{Convex, ConvexHull3D2D, ConvexHullId, FaceIdType};

const EPS: f32 = 1e-5;

/// Convex Hull of 3d data.
/// A `QuickHull` implement.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(crate = "self::serde")]
pub struct ConvexHull3D<'a, T: BasicFloat, P>
where
    P: Into<[T; 3]> + Clone + Copy + Send + Sync + Index<usize, Output = T>,
{
    pub data: Cow<'a, Vec<P>>,
    pub hulls: ConvexHullId,
}

#[derive(Debug, Clone, Copy)]
struct FacePlane<T: BasicFloat> {
    pub face: FaceIdType,
    pub plane: [T; 4],
    pub removed: bool,
}

impl<'a, T: BasicFloat, P> ConvexHull3D<'a, T, P>
where
    P: 'a + Into<[T; 3]> + Clone + Copy + Send + Sync + Index<usize, Output = T>,
{
    /// Return distance between points;
    #[inline]
    fn distance(a: P, b: P) -> T {
        (0..3).fold(T::zero(), |acc, i| acc + (a[i] - b[i]).powi(2))
    }

    /// Return distance to point to plane.
    #[inline]
    fn distance_to_plane(p: &[T; 3], plane: &[T; 4]) -> T {
        p[0] * plane[0] + p[1] * plane[1] + p[2] * plane[2] + plane[3]
    }

    /// Return plane model from three points. `a`x + `b`y + `c`z + `d` = 0;
    #[inline]
    fn generate_plane(points: &(P, P, P)) -> [T; 4] {
        let d0 = [
            points.1[0] - points.0[0],
            points.1[1] - points.0[1],
            points.1[2] - points.0[2],
        ]
        .normalized();
        let d1 = [
            points.2[0] - points.0[0],
            points.2[1] - points.0[1],
            points.2[2] - points.0[2],
        ]
        .normalized();
        let normal = d0.cross(&d1).normalized();
        [
            normal[0],
            normal[1],
            normal[2],
            -(normal[0] * points.0[0] + normal[1] * points.0[1] + normal[2] * points.0[2]),
        ]
    }

    /// Return distance from point to plane is larger than 0 or not.
    #[inline]
    fn visible(plane: &[T; 4], target: &[T; 3]) -> bool {
        Self::distance_to_plane(target, plane) > T::from(EPS).unwrap()
    }

    /// Return points are not equal and not colinear.
    #[inline]
    fn check_face_valid(&self, edges: &[usize; 3]) -> bool {
        // Same vertex check
        if edges[0] == edges[1] || edges[0] == edges[2] || edges[1] == edges[2] {
            return false;
        }
        // Colinear check
        let d1 = apply_both(
            &self.data[edges[1]].into(),
            &self.data[edges[0]].into(),
            std::ops::Div::div,
        )
        .normalized();
        let d2 = apply_both(
            &self.data[edges[2]].into(),
            &self.data[edges[0]].into(),
            std::ops::Div::div,
        )
        .normalized();
        if apply_both(&d2, &d1, std::ops::Div::div).len() <= T::from(EPS).unwrap() {
            return false;
        }

        true
    }

    /// Return XYZ min max ids.
    #[inline]
    fn find_extremum(&self) -> [usize; 6] {
        let data = &self.data;
        let mut extremum = [0usize; 6];
        let mut extremum_value = [
            data[0][0], data[0][1], data[0][2], data[0][0], data[0][1], data[0][2],
        ];
        (0..data.len()).for_each(|i| {
            (0..3).for_each(|ii| {
                if data[i][ii] < extremum_value[ii] {
                    extremum_value[ii] = data[i][ii];
                    extremum[ii] = i;
                }
                if data[i][ii] > extremum_value[ii + 3] {
                    extremum_value[ii + 3] = data[i][ii];
                    extremum[ii + 3] = i;
                }
            });
        });
        extremum
    }

    /// Return largest distance between extremum.
    #[inline]
    fn find_first_edge(&self, ids: &[usize]) -> Line {
        let mut farthest_pair: Line = (0, 0);
        let mut farthest_value = T::min_value();
        (0..6).for_each(|i| {
            (i + 1..6).for_each(|ii| {
                let d = Self::distance(self.data[ids[i]], self.data[ids[ii]]);
                if d > farthest_value {
                    farthest_value = d;
                    farthest_pair = (ids[i], ids[ii]);
                }
            });
        });
        farthest_pair
    }

    /// Return farthest id to first_edge.
    #[inline]
    fn find_third_point(&self, edge: &Line, ids: &[usize]) -> usize {
        let mut third_one = 0usize;
        let mut farthest_value = T::min_value();

        let line: ([T; 3], [T; 3]) = (
            self.data[edge.0].into(),
            [
                self.data[edge.1][0] - self.data[edge.0][0],
                self.data[edge.1][1] - self.data[edge.0][1],
                self.data[edge.1][2] - self.data[edge.0][2],
            ]
            .normalized(),
        );

        (0..ids.len())
            .filter(|&i| ids[i] != edge.0 && ids[i] != edge.1)
            .for_each(|i| {
                let p: [T; 3] = self.data[ids[i]].into();
                let p_dir = [p[0] - line.0[0], p[1] - line.0[1], p[2] - line.0[2]];
                let d = p_dir.cross(&line.1).len();
                if d > farthest_value {
                    farthest_value = d;
                    third_one = ids[i];
                }
            });

        third_one
    }

    /// Return farthest id to plane.
    #[inline]
    fn find_farthest_to_plane(&self, ids: &[usize], plane: &[T; 4]) -> Option<usize> {
        let mut farthest_one = None;
        let mut farthest_value = T::zero();
        ids.iter().for_each(|&i| {
            let d = Self::distance_to_plane(&self.data[i].into(), plane).abs();
            if d <= T::from(EPS).unwrap() {
                return;
            }
            if d > farthest_value {
                farthest_value = d;
                farthest_one = Some(i);
            }
        });
        farthest_one
    }

    /// Return a face oriented to outside.
    ///
    /// * None: three points are equal or colinear.
    #[inline]
    fn generate_face(&self, points: &[usize; 3], target: &[T; 3]) -> Option<FacePlane<T>> {
        if !self.check_face_valid(points) {
            return None;
        }
        let plane = Self::generate_plane(&(
            self.data[points[0]],
            self.data[points[1]],
            self.data[points[2]],
        ));
        if !Self::visible(&plane, target) {
            Some(FacePlane {
                face: FaceIdType { point: *points },
                plane,
                removed: false,
            })
        } else {
            Some(FacePlane {
                face: FaceIdType {
                    point: [points[0], points[2], points[1]],
                },
                plane: [-plane[0], -plane[1], -plane[2], -plane[3]],
                removed: false,
            })
        }
    }

    /// Return ids of faces which point `P` could see (distance from p to plane is larger than 0).
    fn find_visible_faces(faces: &Vec<(FacePlane<T>, Vec<usize>)>, p: &[T; 3]) -> Vec<usize> {
        (0..faces.len())
            .filter(|&i| {
                if faces[i].0.removed {
                    return false;
                }
                Self::visible(&faces[i].0.plane, p)
            })
            .collect()
    }

    /// Return list of edges.
    ///
    /// Iterate all visible faces, find non-overlap edges.
    fn find_hole_edge(faces: &Vec<(FacePlane<T>, Vec<usize>)>, selected: &[usize]) -> Vec<Line> {
        let mut edges_map = HashMap::new();
        (0..faces.len())
            .filter(|i| selected.contains(i))
            .for_each(|i| {
                let (
                    FacePlane {
                        face:
                            FaceIdType {
                                point: [p0, p1, p2],
                            },
                        ..
                    },
                    _,
                ) = faces[i];
                [(p0, p1), (p0, p2), (p1, p2)]
                    .into_iter()
                    .for_each(|(a, b)| {
                        let edge = if a < b { (a, b) } else { (b, a) };
                        let count = edges_map.entry(edge).or_insert(0_usize);
                        *count += 1;
                    });
            });

        edges_map
            .into_iter()
            .filter(|&(_, v)| v <= 1)
            .map(|(k, _)| k)
            .collect()
    }

    /// Expend first hull which has outlier.
    ///
    /// 1. Find the first non-removed face and outliers not empty one. If not match, return;
    /// 2. Find the farthest point (`P`) of outliers to this face. If not match return.
    /// 3. Find faces which `P` could see.
    /// 4. Mark visible faces to removed and collect outliers to a list (`L`).
    /// 5. Find edges of visible faces.
    /// 6. Link edges and `P` as new face, and classify (`L`) to new faces.
    /// 7. Push new faces to `face_set` list.
    fn expend_hull(&self, mid: &[T; 3], face_set: &mut Vec<(FacePlane<T>, Vec<usize>)>) {
        let mut new_face = Vec::with_capacity(face_set.len());
        let mut id = None;
        // Select First face which has outlier points and non-removed face.
        for (i, face) in face_set.iter().enumerate() {
            // Ignore removed face.
            if face.0.removed {
                continue;
            }
            let (_, outliers) = &face;
            // Check has outlier.
            if outliers.is_empty() {
                continue;
            }
            id = Some(i);
            break;
        }
        // Check id is valid.
        let i = match id {
            Some(i) => i,
            None => return,
        };
        // If outlier is on the plane or is the corner, return.
        let farthest = match self.find_farthest_to_plane(&face_set[i].1, &face_set[i].0.plane) {
            Some(farthest) => farthest,
            None => return,
        };
        // Find all faces which farthest point could see.
        let visible = Self::find_visible_faces(face_set, &self.data[farthest].into());
        // Mark all visible faces to removed, and collect outlier points.
        let mut outliers = vec![];
        {
            visible.iter().for_each(|&id_to_remove| {
                let (face, outlier) = &mut face_set[id_to_remove];
                face.removed = true;
                outlier.iter().for_each(|u| outliers.push(*u));
            });
        }
        // Find the edges of ring.
        let hole_edges = Self::find_hole_edge(face_set, &visible);

        let mut outlier_set = HashSet::with_capacity(outliers.len());
        hole_edges.into_iter().for_each(|(e0, e1)| {
            // Link farthest point to each edge as triangle face.
            let new_triangle = match self.generate_face(&[e0, e1, farthest], mid) {
                Some(new_triangle) => new_triangle,
                None => return,
            };
            // Classify outlier points to sub faces.
            let sub_outlier = outliers
                .iter()
                .filter_map(|&o| {
                    if outlier_set.contains(&o) {
                        return None;
                    }
                    if Self::visible(&new_triangle.plane, &self.data[o].into()) {
                        outlier_set.insert(o);
                        Some(o)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            new_face.push((new_triangle, sub_outlier));
        });
        // Add generated faces last, to avoid re-borrow.
        new_face.into_iter().for_each(|h| face_set.push(h));
    }

    /// 1. Compute 4 faces of tetrahedron and orientation.
    /// 2. Put 4 faces to list.
    /// 3. Loop: [`ConvexHull3D::expend_hull`], and remove faces which mark as removed. If all faces have no outside, finish.
    fn compute_3d(&mut self, tetrahedron: [usize; 4]) {
        let [edge_0, edge_1, third, fourth] = tetrahedron;

        let mid = [edge_0, edge_1, third, fourth]
            .into_iter()
            .fold([T::zero(); 3], |acc, i| {
                [
                    acc[0] + self.data[i][0] * T::from(0.25f32).unwrap(),
                    acc[1] + self.data[i][1] * T::from(0.25f32).unwrap(),
                    acc[2] + self.data[i][2] * T::from(0.25f32).unwrap(),
                ]
            });

        let face1 = self.generate_face(&[edge_0, edge_1, third], &mid).unwrap();
        let face2 = self.generate_face(&[edge_0, edge_1, fourth], &mid).unwrap();
        let face3 = self.generate_face(&[edge_0, third, fourth], &mid).unwrap();
        let face4 = self.generate_face(&[edge_1, third, fourth], &mid).unwrap();

        let mut points_set = HashSet::with_capacity(self.data.len());

        let mut hulls = [face1, face2, face3, face4]
            .into_iter()
            .map(|face| {
                let plane = face.plane;
                let outlier = (0..self.data.len())
                    .filter(|&i| {
                        if points_set.contains(&i) {
                            return false;
                        }
                        if Self::visible(&plane, &self.data[i].into()) {
                            points_set.insert(i);
                            true
                        } else {
                            false
                        }
                    })
                    .collect::<Vec<_>>();
                (face, outlier)
            })
            .collect::<Vec<_>>();

        loop {
            self.expend_hull(&mid, &mut hulls);
            hulls.retain(|h| !h.0.removed);

            if hulls.iter().all(|(_, outliers)| outliers.is_empty()) {
                break;
            }
        }

        self.hulls = ConvexHullId::D3(
            hulls
                .into_iter()
                .filter_map(|(h, _)| if !h.removed { Some(h.face) } else { None })
                .collect(),
        );
    }

    /// Compute 2D Convex using [`ConvexHull3D2D`]
    fn compute_2d(&mut self) {
        let mut cvh = ConvexHull3D2D::with_data(&self.data);
        cvh.compute();
        self.hulls = cvh.hulls;
    }
}

impl<'a, T: BasicFloat, P> Convex<'a, P> for ConvexHull3D<'a, T, P>
where
    P: Into<[T; 3]> + Clone + Copy + Send + Sync + Index<usize, Output = T>,
{
    fn new() -> Self {
        Self {
            data: Cow::Owned(vec![]),
            hulls: ConvexHullId::D3(vec![]),
        }
    }

    fn with_data(data: &'a Vec<P>) -> Self {
        Self {
            data: Cow::Borrowed(data),
            hulls: ConvexHullId::D3(vec![]),
        }
    }

    fn set_data(&mut self, data: &'a Vec<P>) {
        self.data = Cow::Borrowed(data);
    }

    /// 1. Find extremum, 6 ids.
    /// 2. Find first edge.
    /// 3. Find farthest point to first edge.
    /// 4. Find farthest point to first plane -> `Option<usize>`.
    ///     * Some(id): id as top point of tetrahedron.
    ///     * None: Data is near a plane, align cloud to XY then compute 2D.
    fn compute(&mut self) {
        let data = &self.data;

        // Get Min Max in three dimension.
        let extremum = self.find_extremum();
        // Get a largest pair between points.
        let edge = self.find_first_edge(&extremum);
        // Should not be the same point.
        assert!(edge.0 != edge.1);
        // Find the farthest one to line model.
        let third = self.find_third_point(&edge, &extremum);
        // Find Tetrahedron the fourth point.
        let plane = Self::generate_plane(&(data[edge.0], data[edge.1], data[third]));
        match self.find_farthest_to_plane(&(0..self.data.len()).collect::<Vec<_>>(), &plane) {
            Some(fourth) => self.compute_3d([edge.0, edge.1, third, fourth]),
            // Could not happen, cause check near plane before.
            None => self.compute_2d(),
        };
    }
}
