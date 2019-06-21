use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Eq, Debug, PartialEq, Clone, Hash)] // FIXME
pub enum NodeState {
    Online,
    Offline,
    Uncontacted,
}

#[derive(PartialEq, Eq, Debug, Clone)] // FIXME
pub struct Node {
    pub addr: SocketAddr,
    pub state: NodeState,
    pub last_visit: SystemTime,
}

pub struct NodeDb {
    nodes: HashMap<SocketAddr, Node>,
}

impl NodeDb {
    pub fn new() -> NodeDb {
        let mut nodes: HashMap<SocketAddr, Node> = HashMap::new();
        NodeDb { nodes }
    }
    pub fn report(&self) -> HashMap<NodeState, i32> {
        let mut report: HashMap<NodeState, i32> = HashMap::new();

        // initialize here so we know all keys always present
        report.insert(NodeState::Online, 0);
        report.insert(NodeState::Offline, 0);
        report.insert(NodeState::Uncontacted, 0);

        // acquire the db lock and build up the report map
        for (_, node) in self.nodes.iter() {
            let mut count = report.entry(node.state.clone()).or_insert(0); // FIXME already initialized ...
            *count += 1; // why?
        }
        report
    }
    // get `n` online nodes (TODO iterate more randomly?)
    pub fn fetch_online_nodes(&self, n: usize) -> Vec<Node> {
        let mut nodes = vec![];
        for (_, node) in self.nodes.iter() {
            if node.state == NodeState::Online {
                nodes.push(node.clone());
                if nodes.len() == n {
                    break;
                }
            }
        }
        nodes
    }
    // get next `n` nodes due for a visit, sets `last_visit` to now
    pub fn next(&mut self) -> Option<Node> {
        let now = SystemTime::now();
        let one_hour_ago = now - Duration::new(60 * 60, 0);
        for (_, node) in &mut self.nodes.iter() {
            if node.last_visit < one_hour_ago {
                let mut n = node.clone();
                n.last_visit = SystemTime::now();
                self.insert(n.clone());
                return Some(n);
            }
        }
        None
    }
    // maybe this should be call "update"
    pub fn insert(&mut self, node: Node) {
        self.nodes.insert(node.addr, node);
    }
    pub fn init(&mut self, addr: SocketAddr) {
        if !self.nodes.contains_key(&addr) {
            self.nodes.insert(
                addr,
                Node {
                    addr: addr,
                    state: NodeState::Uncontacted,
                    last_visit: UNIX_EPOCH,
                },
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report() {
        let n1 = Node {
            addr: "1.1.1.1:8888".parse().unwrap(),
            state: NodeState::Online,
            last_visit: SystemTime::now(),
        };
        let n2 = Node {
            addr: "2.2.2.2:8888".parse().unwrap(),
            state: NodeState::Online,
            last_visit: SystemTime::now(),
        };
        let n3 = Node {
            addr: "3.3.3.3:8888".parse().unwrap(),
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

        // insert one node that isn't due for visit
        let n1 = Node {
            addr: "123.123.123.123:8888".parse().unwrap(),
            state: NodeState::Online,
            last_visit: SystemTime::now(),
        };
        db.insert(n1);
        // n1 isn't due so None is our answer
        assert_eq!(None, db.next());

        // insert another node that is due for a visit
        let n2 = Node {
            addr: "123.123.123.123:8888".parse().unwrap(),
            state: NodeState::Online,
            last_visit: SystemTime::now() - Duration::new(15 * 60, 0),
        };
        db.insert(n2.clone());
        // n2 is due, so is "next"
        let r = db.next().unwrap();
        assert_eq!(n2.addr, r.addr);
        assert!(r.last_visit > SystemTime::now() - Duration::new(1, 0));
    }
}
