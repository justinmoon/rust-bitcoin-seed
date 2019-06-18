use std::clone::Clone;
use std::cmp::{Eq, PartialEq};
use std::collections::HashMap;
use std::hash::Hash;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

#[derive(Eq, PartialEq, Clone, Hash)]
enum NodeState {
    Online,
    Offline,
    Uncontacted,
}

struct Node {
    addr: SocketAddr,
    state: NodeState,
    last_visit: SystemTime,
}

struct NodeDb {
    nodes: Arc<Mutex<Vec<Node>>>,
}

impl NodeDb {
    fn new() -> NodeDb {
        let mut nodes: Vec<Node> = vec![];
        NodeDb {
            nodes: Arc::new(Mutex::new(nodes)),
        }
    }
    fn counts(&self) -> HashMap<NodeState, i32> {
        // HACK can't use NodeState as HashMap key
        let mut counts: HashMap<NodeState, i32> = HashMap::new();

        // initialize here so we know all keys always present
        counts.insert(NodeState::Online, 0);
        counts.insert(NodeState::Offline, 0);
        counts.insert(NodeState::Uncontacted, 0);

        // acquire the db lock and build up the counts map
        let nodes = self.nodes.lock().unwrap();
        for node in nodes.iter() {
            let mut count = counts.entry(node.state.clone()).or_insert(0); // FIXME already initialized ...
            *count += 1; // why?
        }
        counts
    }
    // get next `n` nodes due for a visit
    //fn next(&self) -> Node {}
    fn insert(&self, node: Node) {
        let mut nodes = self.nodes.lock().unwrap();
        nodes.push(node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counts() {
        let n1 = Node {
            addr: "123.123.123.123:8888".parse().unwrap(),
            state: NodeState::Online,
            last_visit: SystemTime::now(),
        };
        let n2 = Node {
            addr: "123.123.123.123:8888".parse().unwrap(),
            state: NodeState::Online,
            last_visit: SystemTime::now(),
        };
        let n3 = Node {
            addr: "123.123.123.123:8888".parse().unwrap(),
            state: NodeState::Offline,
            last_visit: SystemTime::now(),
        };

        let mut db = NodeDb::new();
        db.insert(n1);
        db.insert(n2);
        db.insert(n3);
        let counts = db.counts();
        assert_eq!(2, *counts.get(&NodeState::Online).unwrap());
        assert_eq!(1, *counts.get(&NodeState::Offline).unwrap());
        assert_eq!(0, *counts.get(&NodeState::Uncontacted).unwrap());
    }
}

pub fn crawl() {}
