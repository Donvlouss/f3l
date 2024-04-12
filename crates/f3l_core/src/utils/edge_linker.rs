use std::collections::{HashMap, HashSet};

pub struct EdgeLinker<'a> {
    pub edges: &'a [(usize, usize)],
    pub opened: Vec<Vec<(usize, usize)>>,
    pub closed: Vec<Vec<(usize, usize)>>,
}

struct ShapeDetail<'a> {
    pub edges: &'a [(usize, usize)],
    pub map: HashMap<(usize, usize), usize>,
    pub set: HashSet<usize>,
}

impl<'a> ShapeDetail<'a> {
    pub fn new(edges: &'a [(usize, usize)]) -> Self {
        let mut map = HashMap::new();
        let mut set = HashSet::new();
        edges.iter().for_each(|&e| {
            let entry = map.entry(
                    if e.0 < e.1 {(e.0, e.1)} else {(e.1, e.0)}
                ).or_insert(0_usize);
            *entry += 1;
            set.insert(e.0);
            set.insert(e.1);
        });
        
        Self { edges, map, set }
    }
}

impl<'a> EdgeLinker<'a> {
    pub fn new(edges: &'a [(usize, usize)]) -> Self {
        Self {
            edges,
            opened: Vec::with_capacity(edges.len()),
            closed: Vec::with_capacity(edges.len()),
        }
    }

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

    fn split_open_close(&self, search_opened: bool) -> (Vec<(usize, usize)>, Vec<(usize, usize)>) {
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
            let opened_pts = linked_points.iter().filter_map(|(&k, &v)| {
                if v == 1 {
                    Some(k)
                } else {
                    None
                }
            }).collect::<Vec<_>>();
            opened_pts.iter().for_each(|&p| {
                linked_points.remove_entry(&p);
                for i in 0..self.edges.len() {
                    let (e0, e1) = self.edges[i];
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
        let edges = self.edges.iter().filter_map(|&(e0, e1)| {
            if !linked_points.contains_key(&e0) || !linked_points.contains_key(&e1) {
                if search_opened {
                    opened.push((e0, e1));
                }
                None
            } else {
                Some((e0, e1))
            }
        }).collect::<Vec<_>>();

        (edges, opened)
    }

    fn tear_down_large_shape(&mut self, closed: Vec<Vec<(usize, usize)>>) -> Vec<Vec<(usize, usize)>> {
        let mut closed = closed;

        let mut generated = vec![];
        let mut pointer = 0_usize;
        loop {
            if pointer >= closed.len() {
                break;
            }
            closed.sort_by(|a, b| b.len().partial_cmp(&a.len()).unwrap());
            let i = pointer;
            

            let mut edges = HashMap::new();
            let mut set = HashSet::new();
            closed[i].iter().for_each(|&e| {
                let entry = edges.entry(
                        if e.0 < e.1 {(e.0, e.1)} else {(e.1, e.0)}
                    ).or_insert(0_usize);
                *entry += 1;
                set.insert(e.0);
                set.insert(e.1);
            });


            let mut partial_closed = vec![];
            Self::tear_down_recursive(&closed, i, &edges, &set, &mut partial_closed);
            partial_closed.into_iter().for_each(|e| generated.push(e));

            pointer += 1;
        }

        let mut non_duplicated = HashSet::new();
        generated = generated.into_iter().filter_map(|shape| {
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
        }).collect();

        generated
    }

    fn tear_down_recursive(
        closed: &[Vec<(usize, usize)>],
        start: usize,
        edges: &HashMap<(usize, usize), usize>,
        vertices: &HashSet<usize>,
        partial_closed: &mut Vec<Vec<(usize, usize)>>)
    {
        let mut per_closed = vec![];
        (start+1..closed.len()).rev().for_each(|ii| {
            if closed[ii].iter().any(|j| !vertices.contains(&j.0) || !vertices.contains(&j.1)) {
                return;
            }
            
            let mut edge_map = edges.clone();
            closed[ii].iter().for_each(|&e| {
                let entry = edge_map.entry(
                    if e.0 < e.1 {(e.0, e.1)} else {(e.1, e.0)}
                ).or_insert(0_usize);
                *entry += 1;
            });
            
            let non_common = edge_map.into_iter().filter_map(|(k, v)| {
                if v <= 1 {
                    Some(k)
                } else {
                    None
                }
            }).collect::<Vec<_>>();
            Self::search_closed(&non_common, &mut per_closed);
        });

        if per_closed.is_empty() {
            let shape = edges.into_iter().map(|(&k, _)| {k}).collect::<Vec<_>>();
            partial_closed.push(shape);
            return;
        }
        
        per_closed.into_iter().for_each(|per| {
            let mut map = HashMap::new();
            let mut set = HashSet::new();
            per.iter().for_each(|&e| {
                let entry = map.entry(
                    if e.0 < e.1 {(e.0, e.1)} else {(e.1, e.0)}
                ).or_insert(0_usize);
                *entry += 1;
                set.insert(e.0);
                set.insert(e.1);
            });
            Self::tear_down_recursive(closed, start, &map, &set, partial_closed);
        });

    }

    fn search_opened(opened: &[(usize, usize)], wires: &mut Vec<Vec<(usize, usize)>>) {
        let mut linked = vec![false; opened.len()];
    
        (0..opened.len()).for_each(|i| {
    
            if linked[i] { return; }
    
            let mut temp = vec![opened[i]];
            let mut temp_list = vec![i];
            let mut nb = temp.len();
    
            loop {
                Self::search_recursive(&opened, &mut linked, &mut temp, &mut temp_list, &mut vec![]);
                if nb == temp.len() {
                    break;
                }
                nb = temp.len();
            }
            temp_list.into_iter().for_each(|ii| linked[ii]=true);
    
            wires.push(temp);
        });
    }

    fn search_closed(closed: &[(usize, usize)], wires: &mut Vec<Vec<(usize, usize)>>) {
        let mut visited_edge = vec![false; closed.len()];

        for i in 0..closed.len() {
            if visited_edge[i] {
                continue;
            }
    
            let mut temp = vec![closed[i]];
            let mut temp_list = vec![i];
            while temp[0].0 != temp[temp.len()-1].1 {
                Self::search_recursive(&closed, &mut visited_edge, &mut temp, &mut temp_list, wires);
            }
        }
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
    let mut solver = EdgeLinker::new(&edges);
    solver.search(true);

    println!("Opened: {:?}", solver.opened);
    println!("Closed: {:?}", solver.closed);
}