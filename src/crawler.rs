use bitcoin::consensus::encode::serialize;
use bitcoin::network::constants::Network;
use bitcoin::network::{
    address::Address,
    message::{NetworkMessage, RawNetworkMessage},
    message_network::VersionMessage,
    stream_reader::StreamReader,
};
use log::{info, trace};
use std::io::Write;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use super::db;
use super::utils;

fn bootstrap(tdb: Arc<Mutex<db::NodeDb>>) {
    let mut db = tdb.lock().unwrap();
    for addr in utils::dns_seed(Network::Bitcoin) {
        db.init(addr);
    }
}

fn visit(node: db::Node) -> Result<WorkerOutput, utils::CrawlerError> {
    trace!("Connecting to {}", &node.addr);
    let mut worker_output = WorkerOutput::new(node.clone());
    let mut stream = TcpStream::connect_timeout(&node.addr, Duration::new(1, 0))?;
    trace!("Connected to {}", &node.addr);

    // timeout in 30 seconds
    stream.set_read_timeout(Some(Duration::new(5, 0)))?;

    // write version
    let lversion = utils::compile_version();
    stream.write(&serialize(&RawNetworkMessage {
        magic: 0xd9b4bef9,
        payload: lversion,
    }))?;
    trace!("Sent version");

    // handle messages as they arrive
    loop {
        let mut reader = StreamReader::new(&mut stream, Some(10000000));
        match reader.next_message() {
            Ok(msg) => match msg.payload {
                NetworkMessage::Version(ref rversion) => {
                    trace!("Received version");
                    let lverack = NetworkMessage::Verack;
                    stream.write(&serialize(&RawNetworkMessage {
                        magic: 0xd9b4bef9,
                        payload: lverack,
                    }))?;
                    worker_output.version_msg = Some(rversion.clone());
                    trace!("Sent verack");
                }
                NetworkMessage::Verack => {
                    trace!("Received verack");
                    let getaddr = NetworkMessage::GetAddr;
                    stream.write(&serialize(&RawNetworkMessage {
                        magic: 0xd9b4bef9,
                        payload: getaddr,
                    }))?;
                    trace!("Sent getaddr");
                }
                NetworkMessage::Ping(ref ping) => {
                    trace!("Received ping");
                    let pong = NetworkMessage::Pong(*ping);
                    stream.write(&serialize(&RawNetworkMessage {
                        magic: 0xd9b4bef9,
                        payload: pong,
                    }))?;
                    trace!("Sent pong");
                }
                NetworkMessage::Addr(ref addr) => {
                    trace!("Received {} addrs", addr.len());
                    if addr.len() > 1 {
                        worker_output.addr_msg = Some(addr.clone());
                        break;
                    }
                }
                _ => {
                    trace!("Received {}", msg.command());
                }
            },
            Err(err) => {
                trace!("P2P error: {}", err.to_string());
                let fatal_errors = vec![
                    // stream timed out
                    String::from("Resource temporarily unavailable (os error 11)"),
                    // peer hung up (?)
                    String::from("unexpected end of file"),
                    // peer hung up (?)
                    String::from("invalid checksum: expected 5df6e0e2, actual 00000000"),
                ];
                if fatal_errors.contains(&err.to_string()) {
                    break;
                }
            }
        }
    }

    return Ok(worker_output);
}

struct WorkerOutput {
    node: db::Node,
    version_msg: Option<VersionMessage>,
    addr_msg: Option<Vec<(u32, Address)>>,
}

impl WorkerOutput {
    fn new(node: db::Node) -> WorkerOutput {
        WorkerOutput {
            node: node,
            version_msg: None,
            addr_msg: None,
        }
    }
}

fn worker(tdb: Arc<Mutex<db::NodeDb>>) {
    loop {
        let mut db = tdb.lock().unwrap();
        let next = db.next();
        drop(db);
        // if next, visit them. otherwise, sleep.
        let result = match next {
            Some(node) => visit(node),

            None => {
                thread::sleep(Duration::new(1 * 60, 0));
                break;
            }
        };
        // if `version_msg` present in output, mark node online. otherwise,
        // mark them offline
        match result {
            Ok(mut output) => {
                match output.version_msg {
                    Some(_) => {
                        output.node.state = db::NodeState::Online;
                        let mut db = tdb.lock().unwrap();
                        db.insert(output.node);
                    }
                    None => {
                        output.node.state = db::NodeState::Offline;
                        let mut db = tdb.lock().unwrap();
                        db.insert(output.node);
                    }
                }
                // if addr_msg present on `output`, initialize these records in db
                match output.addr_msg {
                    Some(addr_msg) => {
                        for net_addr in addr_msg {
                            match net_addr.1.socket_addr() {
                                Ok(addr) => {
                                    let mut db = tdb.lock().unwrap();
                                    db.init(addr);
                                }
                                Err(_) => (),
                            }
                        }
                    }
                    None => (),
                }
            }
            Err(err) => trace!("Crawler error: {}", err),
        }
    }
}

fn spawn(nthreads: i32, tdb: Arc<Mutex<db::NodeDb>>) {
    log::info!("Starting {} threads", nthreads);
    for i in 0..nthreads {
        let db = Arc::clone(&tdb);
        thread::Builder::new()
            .name(format!("thread-{}", i.to_string()))
            .spawn(move || {
                worker(db);
            })
            .expect("Couldn't spawn thread");
    }
}

pub fn crawl() {
    utils::init_logger();
    let db = db::NodeDb::new();
    let tdb = Arc::new(Mutex::new(db));
    bootstrap(tdb.clone());
    spawn(1000, tdb.clone());
    loop {
        thread::sleep(Duration::new(1, 0));
        let _db = tdb.lock().unwrap();
        let report = _db.report();
        info!(
            "Online: {:?} Offline: {:?} Uncontacted {:?}",
            report.get(&db::NodeState::Online).unwrap(),
            report.get(&db::NodeState::Offline).unwrap(),
            report.get(&db::NodeState::Uncontacted).unwrap(),
        );
    }
}
