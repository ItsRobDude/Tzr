use std::collections::{HashMap, HashSet, VecDeque};

use crate::model::RoomId;

/// Deterministic adjacency list for dungeon rooms.
/// Neighbor lists are stored in ascending `RoomId` order to guarantee
/// stable traversal results.
#[derive(Debug, Default, Clone)]
pub struct Adjacency {
    pub neighbors: HashMap<RoomId, Vec<RoomId>>,
}

impl Adjacency {
    /// Build adjacency from an undirected edge list.
    pub fn from_edges(edges: &[(RoomId, RoomId)]) -> Self {
        let mut neighbors: HashMap<RoomId, Vec<RoomId>> = HashMap::new();

        for (a, b) in edges {
            neighbors.entry(*a).or_default().push(*b);
            neighbors.entry(*b).or_default().push(*a);
        }

        for list in neighbors.values_mut() {
            list.sort_by_key(|room| room.0);
            list.dedup();
        }

        Adjacency { neighbors }
    }
}

/// Compute the shortest path between `from` and `to` using BFS.
/// Returns the inclusive sequence of rooms if a path exists.
pub fn shortest_path(graph: &Adjacency, from: RoomId, to: RoomId) -> Option<Vec<RoomId>> {
    if from == to {
        return Some(vec![from]);
    }

    let mut queue = VecDeque::new();
    let mut visited: HashSet<RoomId> = HashSet::new();
    let mut parents: HashMap<RoomId, RoomId> = HashMap::new();

    visited.insert(from);
    queue.push_back(from);

    while let Some(current) = queue.pop_front() {
        if let Some(neigh) = graph.neighbors.get(&current) {
            for &next in neigh {
                if visited.contains(&next) {
                    continue;
                }
                visited.insert(next);
                parents.insert(next, current);

                if next == to {
                    let mut path = vec![to];
                    let mut cursor = to;
                    while let Some(&parent) = parents.get(&cursor) {
                        path.push(parent);
                        if parent == from {
                            break;
                        }
                        cursor = parent;
                    }
                    path.reverse();
                    return Some(path);
                }

                queue.push_back(next);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::{Adjacency, shortest_path};
    use crate::model::RoomId;

    #[test]
    fn finds_single_simple_path() {
        let edges = vec![(RoomId(1), RoomId(2)), (RoomId(2), RoomId(3))];
        let graph = Adjacency::from_edges(&edges);

        let path = shortest_path(&graph, RoomId(1), RoomId(3)).unwrap();

        assert_eq!(path, vec![RoomId(1), RoomId(2), RoomId(3)]);
    }

    #[test]
    fn prefers_shorter_path_when_multiple_exist() {
        // Two possible routes from 1 -> 5. Path through 2 is shorter than path through 3 -> 4.
        let edges = vec![
            (RoomId(1), RoomId(2)),
            (RoomId(2), RoomId(5)),
            (RoomId(1), RoomId(3)),
            (RoomId(3), RoomId(4)),
            (RoomId(4), RoomId(5)),
        ];
        let graph = Adjacency::from_edges(&edges);

        let path = shortest_path(&graph, RoomId(1), RoomId(5)).unwrap();

        assert_eq!(path, vec![RoomId(1), RoomId(2), RoomId(5)]);
    }

    #[test]
    fn deterministic_choice_on_equal_length_paths() {
        // Two equal-length routes: 1-2-4 and 1-3-4. Sorted neighbors should pick 2 first.
        let edges = vec![
            (RoomId(1), RoomId(2)),
            (RoomId(2), RoomId(4)),
            (RoomId(1), RoomId(3)),
            (RoomId(3), RoomId(4)),
        ];
        let graph = Adjacency::from_edges(&edges);

        let path = shortest_path(&graph, RoomId(1), RoomId(4)).unwrap();

        assert_eq!(path, vec![RoomId(1), RoomId(2), RoomId(4)]);
    }
}
