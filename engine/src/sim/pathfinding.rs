use std::collections::{HashMap, HashSet, VecDeque};

use crate::model::RoomId;

/// Compute the shortest path between two rooms using BFS.
/// Returns the sequence of room ids from `start` to `goal`, inclusive.
pub fn shortest_path(
    start: RoomId,
    goal: RoomId,
    edges: &[(RoomId, RoomId)],
) -> Option<Vec<RoomId>> {
    if start == goal {
        return Some(vec![start]);
    }

    let mut queue = VecDeque::new();
    let mut visited: HashSet<RoomId> = HashSet::new();
    let mut parents: HashMap<RoomId, RoomId> = HashMap::new();

    queue.push_back(start);
    visited.insert(start);

    while let Some(current) = queue.pop_front() {
        for neighbor in neighbors(current, edges) {
            if visited.insert(neighbor) {
                parents.insert(neighbor, current);
                if neighbor == goal {
                    return Some(reconstruct_path(goal, &parents));
                }
                queue.push_back(neighbor);
            }
        }
    }

    None
}

fn neighbors(room: RoomId, edges: &[(RoomId, RoomId)]) -> impl Iterator<Item = RoomId> + '_ {
    edges.iter().filter_map(move |(a, b)| {
        if *a == room {
            Some(*b)
        } else if *b == room {
            Some(*a)
        } else {
            None
        }
    })
}

fn reconstruct_path(goal: RoomId, parents: &HashMap<RoomId, RoomId>) -> Vec<RoomId> {
    let mut path = vec![goal];
    let mut current = goal;
    while let Some(parent) = parents.get(&current) {
        path.push(*parent);
        current = *parent;
    }
    path.reverse();
    path
}
