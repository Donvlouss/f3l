use crate::Line;
use std::collections::{HashMap, HashSet};

/// A Solver to find multiple-edges, which could be shapes.
pub struct EdgeLinker<'a> {
    pub edges: &'a [Line],
    pub opened: Vec<Vec<Line>>,
    pub closed: Vec<Vec<Line>>,
}

/// Private Structure to represent continuos edges profile.
///
/// Build map and set when call [`ShapeDetail::new`]
struct ShapeDetail {
    /// To check common-edges.
    pub map: HashMap<Line, usize>,
    /// To Check common-vertices.
    pub set: HashSet<usize>,
}

impl ShapeDetail {
    pub fn new(edges: &[Line]) -> Self {
        let mut map = HashMap::new();
        let mut set = HashSet::new();
        edges.iter().for_each(|&e| {
            let entry = map
                .entry(if e.0 < e.1 { (e.0, e.1) } else { (e.1, e.0) })
                .or_insert(0_usize);
            *entry += 1;
            set.insert(e.0);
            set.insert(e.1);
        });

        Self { map, set }
    }
}

impl<'a> EdgeLinker<'a> {
    pub fn new(edges: &'a [Line]) -> Self {
        Self {
            edges,
            opened: Vec::with_capacity(edges.len()),
            closed: Vec::with_capacity(edges.len()),
        }
    }

    /// Search continuos edges and opened edges.
    ///
    /// 1. Split edges to opened and closed. see [`EdgeLinker::split_open_close`]
    /// 2. If `search_opened` is `true`, [`EdgeLinker::search_opened`]
    /// 3. Search closed, [`EdgeLinker::search_closed`]
    /// 4. Some shape contains other shape, need to tear down, [`EdgeLinker::tear_down_large_shape`]
    ///
    pub fn search(&mut self, search_opened: bool) {
        // Split to closed edges and opened edges by counting linked numbers of vertices.
        let (closed, opened) = self.split_open_close(search_opened);

        // Search opened edges.
        if search_opened {
            Self::search_opened(&opened, &mut self.opened);
        }

        let mut closed_shape = vec![];
        Self::search_closed(&closed, &mut closed_shape);

        self.closed = self.tear_down_large_shape(closed_shape);
    }

    /// Return (closed-edges, opened-edges).
    ///
    /// 1. Build Map to find non-common vertices.
    /// 2. Recursive loop Map of non-common vertices, reduce counter of vertex.
    /// 3. If counter is 0, remove this vertex in Map.
    /// 4. Iterate edges to find start and end of edge are all in Map.
    /// 5. If vertices of edge are all in Mpa, add to closed, else add to opened.
    fn split_open_close(&self, search_opened: bool) -> (Vec<Line>, Vec<Line>) {
        // First search opened edges.
        let mut linked_points = HashMap::new();
        self.edges.iter().for_each(|&(e0, e1)| {
            let e = linked_points.entry(e0).or_insert(0_usize);
            *e += 1;
            let e = linked_points.entry(e1).or_insert(0_usize);
            *e += 1;
        });
        // Recursive remove opened edges.
        while linked_points.values().any(|&count| count == 1) {
            let opened_pts = linked_points
                .iter()
                .filter_map(|(&k, &v)| if v == 1 { Some(k) } else { None })
                .collect::<Vec<_>>();
            opened_pts.iter().for_each(|&p| {
                linked_points.remove_entry(&p);
                for i in 0..self.edges.len() {
                    let (e0, e1) = self.edges[i];
                    let other = if e0 == p {
                        e1
                    } else if e1 == p {
                        e0
                    } else {
                        continue;
                    };
                    match linked_points.get_mut(&other) {
                        Some(entry) => {
                            *entry -= 1;
                            if *entry == 0 {
                                linked_points.remove_entry(&other);
                            }
                        }
                        None => continue,
                    };
                }
            });
        }
        let mut opened = vec![];
        let edges = self
            .edges
            .iter()
            .filter_map(|&(e0, e1)| {
                if !linked_points.contains_key(&e0) || !linked_points.contains_key(&e1) {
                    if search_opened {
                        opened.push((e0, e1));
                    }
                    None
                } else {
                    Some((e0, e1))
                }
            })
            .collect::<Vec<_>>();

        (edges, opened)
    }

    /// Return all non-overlapped edges of shapes.
    ///
    /// 1. Sort closed list to a decreasing list. (The last one must not be overlapped.)
    /// 2. Iterate closed, build [`ShapeDetail`] then call [`EdgeLinker::tear_down_recursive`] to recursive search.
    /// 3. Add all result to a list, then remove duplicated.
    fn tear_down_large_shape(&mut self, closed: Vec<Vec<Line>>) -> Vec<Vec<Line>> {
        let mut closed = closed;
        if closed.is_empty() {
            return vec![];
        }

        let mut generated = vec![];
        closed.sort_by(|a, b| b.len().partial_cmp(&a.len()).unwrap());

        (0..closed.len() - 1).for_each(|i| {
            let mut partial_closed = vec![];
            let profile = ShapeDetail::new(&closed[i]);
            Self::tear_down_recursive(&closed, i, profile, &mut partial_closed);
            partial_closed.into_iter().for_each(|e| generated.push(e));
        });
        generated.push(closed[closed.len() - 1].clone());

        let mut non_duplicated = HashSet::new();
        generated = generated
            .into_iter()
            .filter_map(|shape| {
                let mut set = HashSet::new();
                shape.iter().for_each(|&e| {
                    set.insert(e.0);
                    set.insert(e.1);
                });
                let mut set = set.into_iter().collect::<Vec<_>>();
                set.sort();
                if non_duplicated.contains(&set) {
                    None
                } else {
                    non_duplicated.insert(set);
                    Some(shape)
                }
            })
            .collect();

        generated
    }

    /// Tear-down recursive method.
    ///
    /// 1. Iterate to find contour which all vertices is overlap the target one.
    /// 2. Find non-common edges., then [`EdgeLinker::search_closed`].
    /// 3. If result of [`EdgeLinker::search_closed`] is empty, means this is a closed, push to `partial_closed`, return.
    /// 4. Iterate the result of [`EdgeLinker::search_closed`], recursive this.
    fn tear_down_recursive(
        closed: &[Vec<Line>],
        start: usize,
        profile: ShapeDetail,
        partial_closed: &mut Vec<Vec<Line>>,
    ) {
        let mut per_closed = vec![];
        (start + 1..closed.len()).rev().for_each(|ii| {
            if closed[ii]
                .iter()
                .any(|j| !&profile.set.contains(&j.0) || !&profile.set.contains(&j.1))
            {
                return;
            }

            let mut edge_map = profile.map.clone();
            closed[ii].iter().for_each(|&e| {
                let entry = edge_map
                    .entry(if e.0 < e.1 { (e.0, e.1) } else { (e.1, e.0) })
                    .or_insert(0_usize);
                *entry += 1;
            });

            let non_common = edge_map
                .into_iter()
                .filter_map(|(k, v)| if v <= 1 { Some(k) } else { None })
                .collect::<Vec<_>>();
            Self::search_closed(&non_common, &mut per_closed);
        });

        if per_closed.is_empty() {
            let shape = profile.map.into_iter().map(|(k, _)| k).collect::<Vec<_>>();
            partial_closed.push(shape);
            return;
        }

        per_closed.into_iter().for_each(|per| {
            let mut map = HashMap::new();
            let mut set = HashSet::new();
            per.iter().for_each(|&e| {
                let entry = map
                    .entry(if e.0 < e.1 { (e.0, e.1) } else { (e.1, e.0) })
                    .or_insert(0_usize);
                *entry += 1;
                set.insert(e.0);
                set.insert(e.1);
            });
            let profile = ShapeDetail::new(&per);
            Self::tear_down_recursive(closed, start, profile, partial_closed);
        });
    }

    /// Search opened-edges.
    ///
    /// Recursive [`EdgeLinker::search_recursive`], if `temp` is the same number between before and after,
    /// means search end.
    fn search_opened(opened: &[Line], wires: &mut Vec<Vec<Line>>) {
        let mut linked = vec![false; opened.len()];

        (0..opened.len()).for_each(|i| {
            if linked[i] {
                return;
            }

            let mut temp = vec![opened[i]];
            let mut temp_list = vec![i];
            let mut nb = temp.len();

            loop {
                Self::search_recursive(opened, &mut linked, &mut temp, &mut temp_list, &mut vec![]);
                if nb == temp.len() {
                    break;
                }
                nb = temp.len();
            }
            temp_list.into_iter().for_each(|ii| linked[ii] = true);

            wires.push(temp);
        });
    }

    /// Search closed-edge.
    ///
    /// Recursive [`EdgeLinker::search_recursive`], if `temp.first.start` == `temp.last.end`,
    /// means search end.
    fn search_closed(closed: &[Line], wires: &mut Vec<Vec<Line>>) {
        let mut visited_edge = vec![false; closed.len()];

        for i in 0..closed.len() {
            if visited_edge[i] {
                continue;
            }

            let mut temp = vec![closed[i]];
            let mut temp_list = vec![i];
            while temp[0].0 != temp[temp.len() - 1].1 {
                Self::search_recursive(closed, &mut visited_edge, &mut temp, &mut temp_list, wires);
            }
        }
    }

    /// A Search Recursive method.
    ///
    /// 1. Iterate edges.
    /// 2. Find index not in `temp_list` and not (start, end) of edge is not equal to current or is not matched previous.
    /// 3. If `temp.len()` is 1, just push, cause a closed-shape contains at least 3 edges.
    /// 4. Reverse search `id` which could be a closed-shape.
    /// 5. If has `id`, update `temp`, `temp_list`, `visited`, and end, else push this to temp.
    fn search_recursive(
        edges: &[Line],
        visited: &mut [bool],
        temp: &mut Vec<Line>,
        temp_list: &mut Vec<usize>,
        closed: &mut Vec<Vec<Line>>,
    ) {
        for i in 0..edges.len() {
            // visited
            if temp_list.contains(&i) {
                continue;
            }
            let (previous, current) = temp[temp.len() - 1];

            let (e0, e1) = edges[i];
            // edge not connect current temp wire
            if e0 != current && e1 != current {
                continue;
            }
            if e0 == previous || e1 == previous {
                continue;
            }
            let next = if e0 == current { (e0, e1) } else { (e1, e0) };
            // Only one edge, just push
            if temp.len() == 1 {
                temp.push(next);
                temp_list.push(i);
                continue;
            }
            // Reverse search temp list.
            let mut id = None;
            for ii in (0..=temp.len() - 2).rev() {
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
        (8, 10),
    ];
    let mut solver = EdgeLinker::new(&edges);
    solver.search(true);

    let mut opened = vec![
        vec![(0_usize, 1_usize)],
        vec![(3, 4), (4, 5)],
        vec![(8, 9)],
        vec![(8, 10)],
    ];
    let mut closed = vec![
        vec![(2, 6), (2, 7), (6, 8), (7, 8)],
        vec![(1, 2), (1, 6), (2, 6)],
        vec![(2, 7), (3, 2), (7, 3)],
    ];
    let mut solver_opened = solver.opened;
    let mut solver_closed = solver.closed;

    // Sort each to make sure indices could be the same.
    for o in opened.iter_mut() {
        o.sort();
    }
    for o in closed.iter_mut() {
        o.sort();
    }
    for o in solver_opened.iter_mut() {
        o.sort();
    }
    for o in solver_closed.iter_mut() {
        o.sort();
    }

    solver_opened.iter().for_each(|e| {
        assert!(opened.contains(e));
    });
    solver_closed.iter().for_each(|e| {
        assert!(closed.contains(e));
    });
}
