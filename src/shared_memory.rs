use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use std::clone::Clone;
use std::cmp::{Eq, PartialEq};
use std::hash::Hash;

#[derive(Eq, Debug, PartialEq, Clone, Hash)]
enum NodeState {
    Online,
    Offline,
    Uncontacted,
}

#[derive(PartialEq, Eq, Debug, Clone)]
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
    fn report(&self) -> HashMap<NodeState, i32> {
        // HACK can't use NodeState as HashMap key
        let mut report: HashMap<NodeState, i32> = HashMap::new();

        // initialize here so we know all keys always present
        report.insert(NodeState::Online, 0);
        report.insert(NodeState::Offline, 0);
        report.insert(NodeState::Uncontacted, 0);

        // acquire the db lock and build up the report map
        let nodes = self.nodes.lock().unwrap();
        for node in nodes.iter() {
            let mut count = report.entry(node.state.clone()).or_insert(0); // FIXME already initialized ...
            *count += 1; // why?
        }
        report
    }
    // get next `n` nodes due for a visit
    fn next(&self) -> Option<Node> {
        let nodes = self.nodes.lock().unwrap();
        let now = SystemTime::now();
        let ten_minutes_ago = now - Duration::new(10 * 60, 0);
        for node in nodes.iter() {
            if node.last_visit < ten_minutes_ago {
                return Some((node.clone()));
            }
        }
        None
    }
    fn insert(&self, node: Node) {
        let mut nodes = self.nodes.lock().unwrap();
        nodes.push(node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report() {
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
        let report = db.report();
        assert_eq!(2, *report.get(&NodeState::Online).unwrap());
        assert_eq!(1, *report.get(&NodeState::Offline).unwrap());
        assert_eq!(0, *report.get(&NodeState::Uncontacted).unwrap());
    }

    #[test]
    fn test_next() {
        let mut db = NodeDb::new();
        let n1 = Node {
            addr: "123.123.123.123:8888".parse().unwrap(),
            state: NodeState::Online,
            last_visit: SystemTime::now(),
        };
        db.insert(n1);
        assert_eq!(None, db.next());
        let n2 = Node {
            addr: "123.123.123.123:8888".parse().unwrap(),
            state: NodeState::Online,
            last_visit: SystemTime::now() - Duration::new(15 * 60, 0),
        };
        db.insert(n2.clone());
        assert_eq!(Some(n2), db.next());
    }
}

pub fn crawl() {}
