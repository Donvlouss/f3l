use std::collections::HashMap;

use f3l_core::{find_circle, get_minmax, EdgeLinker};

use super::{BasicFloat, FaceIdType, SubTriangle};

pub struct Delaunay2D<'a, T: BasicFloat> {
    pub data: &'a [[T; 2]],
    pub triangles: Vec<FaceIdType>,
    pub super_triangle: [[T;2];3],
    pub contours: Vec<Vec<(usize, usize)>>
}


impl<'a, T: BasicFloat> Delaunay2D<'a, T> {
    pub fn new(data: &'a [[T; 2]]) -> Self {
        Self {
            data, 
            triangles: vec![],
            super_triangle: [[T::zero(); 2]; 3],
            contours: vec![]
        }
    }

    #[inline]
    pub fn at(&self, id: usize) -> [T; 2] {
        match id >= self.data.len() {
            true => self.super_triangle[id - self.data.len()],
            false => self.data[id],
        }
    }

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

    pub fn generate_triangle(&self, ids: [usize; 3]) -> SubTriangle<T> {
        let (center, radius) = self.find_out_circle(ids);
        SubTriangle {
            tri: FaceIdType { point: ids },
            removed: false,
            center, radius
        }
    }

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

    pub fn insert(&mut self, id: usize, triangles: &mut Vec<SubTriangle<T>>) -> Vec<SubTriangle<T>> {
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
            
        new_faces
    }

    pub fn compute(&mut self, alpha: T) {
        self.triangles.clear();
        self.contours.clear();

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
        self.triangles = mesh;
        
        // let wires = search_continuos_wire(&contours);
        // self.contours = wires;
        let mut solver = EdgeLinker::new(&contours);
        solver.search(false);
        self.contours = solver.closed;
    }

    pub fn compute_alpha(&self, triangles: &Vec<SubTriangle<T>>, alpha: T) -> (Vec<FaceIdType>, Vec<(usize, usize)>) {
        let mut inner = HashMap::new();
        let inner_triangles = triangles.iter().filter_map(|&face| {
            if face.radius < alpha {
                [(0_usize, 1), (0, 2), (1, 2)].into_iter().for_each(|e| {
                    let (e0, e1) = (face.tri.point[e.0], face.tri.point[e.1]);
                    let edge = if e0 < e1 {(e0, e1)} else {(e1, e0)};
                    let entry = inner.entry(edge).or_insert(0);
                    *entry += 1;
                });
                Some(face.tri)
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
}

// TODO Large shape could contains small shape, need to split.
fn search_continuos_wire(edges: &[(usize, usize)]) -> Vec<Vec<(usize, usize)>> {
    let mut visited_edge = vec![false; edges.len()];
    let mut wires: Vec<Vec<(usize, usize)>>= vec![];

    // Recursive search opened edges.
    let mut linked_points = HashMap::new();
    edges.iter().for_each(|&(e0, e1)| {
        let e = linked_points.entry(e0).or_insert(0_usize);
        *e += 1;
        let e = linked_points.entry(e1).or_insert(0_usize);
        *e += 1;
    });
    while linked_points.values().any(|&count| count == 1) {
        let opened_pts = linked_points.iter().filter_map(|(&k, &v)| {
            if v == 1 {
                Some(k)
            } else {
                None
            }
        }).collect::<Vec<_>>();
        opened_pts.iter().for_each(|&p| {
            linked_points.remove_entry(&p);
            for i in 0..edges.len() {
                let (e0, e1) = edges[i];
                let other = if e0 == p {e1} else if e1 == p {e0} else {continue;};
                match linked_points.get_mut(&other) {
                    Some(entry) => {
                        *entry -= 1;
                        if *entry == 0 {
                            linked_points.remove_entry(&other);
                        }
                    },
                    None => continue,
                };
            }
        });
    }
    let mut opened = vec![];
    let edges = edges.iter().filter_map(|&(e0, e1)| {
        if !linked_points.contains_key(&e0) || !linked_points.contains_key(&e1) {
            opened.push((e0, e1));
            None
        } else {
            Some((e0, e1))
        }
    }).collect::<Vec<_>>();

    search_opened(&opened, &mut wires);

    for i in 0..edges.len() {
        if visited_edge[i] {
            continue;
        }

        let mut temp = vec![edges[i]];
        let mut temp_list = vec![i];
        while temp[0].0 != temp[temp.len()-1].1 {
            search_recursive(&edges, &mut visited_edge, &mut temp, &mut temp_list, &mut wires);
        }
    }
    
    wires
}

fn search_opened(opened: &[(usize, usize)], wires: &mut Vec<Vec<(usize, usize)>>) {
    let mut linked = vec![false; opened.len()];
    let mut opened_wires: Vec<Vec<(usize, usize)>> = vec![];

    (0..opened.len()).for_each(|i| {

        if linked[i] { return; }

        let mut temp = vec![opened[i]];
        let mut temp_list = vec![i];
        let mut nb = temp.len();

        loop {
            search_recursive(&opened, &mut linked, &mut temp, &mut temp_list, &mut opened_wires);
            if nb == temp.len() {
                break;
            }
            nb = temp.len();
        }
        temp_list.into_iter().for_each(|ii| linked[ii]=true);

        wires.push(temp);
    });
}

fn search_recursive(edges: &[(usize, usize)], visited: &mut Vec<bool>, temp: &mut Vec<(usize, usize)>, temp_list: &mut Vec<usize>, closed: &mut Vec<Vec<(usize, usize)>>) {

    for i in 0..edges.len() {
        // visited
        if temp_list.contains(&i) {
            continue;
        }
        let (previous, current) = temp[temp.len()-1];
        
        let (e0, e1) = edges[i];
        // edge not connect current temp wire
        if e0 != current && e1 != current {
            continue;
        }
        if e0 == previous || e1 == previous {
            continue;
        }
        let next = if e0 == current {(e0, e1)} else {(e1, e0)};
        // Only one edge, just push
        if temp.len() == 1 {
            temp.push(next);
            temp_list.push(i);
            continue;
        }
        // Reverse search temp list.
        let mut id = None;
        for ii in (0..=temp.len()-2).rev() {
            if temp[ii].0 == next.1 {
                id = Some(ii);
            }
        }
        if let Some(id) = id {
            (id..temp.len()).for_each(|iii| {
                visited[temp_list[iii]] = true;
            });
            visited[i] = true;
            temp.push(next);
            closed.push(temp.clone());
            break;
        } else {
            temp.push(next);
            temp_list.push(i);
        };
    }
}

#[test]
fn test() {
    let edges = vec![
        (0, 1),
        (1, 2),
        (1, 6),
        (2, 3),
        (2, 6),
        (2, 7),
        (3, 4),
        (3, 7),
        (4, 5),
        (6, 8),
        (7, 8),
        (8, 9),
        (8, 10)
    ];
    println!("{:?}", search_continuos_wire(&edges));
}