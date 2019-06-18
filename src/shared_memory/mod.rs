use bitcoin::consensus::encode::serialize;
use bitcoin::network::{
    address::Address,
    message::{NetworkMessage, RawNetworkMessage},
    message_network::VersionMessage,
    stream_reader::StreamReader,
};
use std::collections::HashMap;
use std::io::Write;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bitcoin::network::constants::Network;

mod db;
mod utils;

fn bootstrap(db: &mut db::NodeDb) {
    for addr in utils::dns_seed(Network::Bitcoin) {
        println!("initialized {}", addr);
        db.initialize(addr);
    }
    println!("finished bootstrapping");
}

fn visit(node: db::Node) -> WorkerOutput {
    println!("Connecting to {}", &node.addr);
    let mut worker_output = WorkerOutput::new(node.clone());
    match TcpStream::connect_timeout(&node.addr, Duration::new(1, 0)) {
        Ok(mut stream) => {
            println!("Connected");

            // timeout in 30 seconds
            stream
                .set_read_timeout(Some(Duration::new(30, 0)))
                .expect("Couldn't set timeout");

            // write version
            let lversion = utils::compile_version();
            stream
                .write(&serialize(&RawNetworkMessage {
                    magic: 0xd9b4bef9,
                    payload: lversion,
                }))
                .expect("Couldn't write version");
            println!("Sent version");

            // handle messages as they arrive
            loop {
                let mut reader = StreamReader::new(&mut stream, Some(10000000));
                match reader.next_message() {
                    Ok(message) => match message.payload {
                        NetworkMessage::Version(ref rversion) => {
                            println!("Received version");
                            let lverack = NetworkMessage::Verack;
                            stream
                                .write(&serialize(&RawNetworkMessage {
                                    magic: 0xd9b4bef9,
                                    payload: lverack,
                                }))
                                .expect("Couldn't write verack");
                            worker_output.version_msg = Some(rversion.clone());
                            println!("Sent verack");
                        }
                        NetworkMessage::Verack => {
                            println!("Received verack");
                            let getaddr = NetworkMessage::GetAddr;
                            stream
                                .write(&serialize(&RawNetworkMessage {
                                    magic: 0xd9b4bef9,
                                    payload: getaddr,
                                }))
                                .expect("Couldn't write getaddr");
                            println!("Sent getaddr");
                        }
                        NetworkMessage::Ping(ref ping) => {
                            println!("Received ping");
                            let pong = NetworkMessage::Pong(*ping);
                            stream
                                .write(&serialize(&RawNetworkMessage {
                                    magic: 0xd9b4bef9,
                                    payload: pong,
                                }))
                                .expect("Couldn't write pong");
                            println!("Sent pong");
                        }
                        NetworkMessage::Addr(ref addr) => {
                            println!("Received {} addrs", addr.len());
                            if addr.len() > 1 {
                                worker_output.addr_msg = Some(addr.clone());
                                break;
                            }
                        }
                        _ => {
                            println!("Received {}", message.command());
                        }
                    },
                    Err(err) => {
                        println!("Error: {}", err.to_string());
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
            return worker_output;
        }
        Err(e) => {
            println!("Failed to connect: {}", e);
            return worker_output;
        }
    }
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

fn worker(db: &mut db::NodeDb) {
    loop {
        // TODO: print how many nodes are due for a visit
        let next = db.next();
        let mut output = match next {
            Some(node) => visit(node),
            None => {
                thread::sleep(Duration::new(1 * 60, 0));
                break;
            }
        };
        match output.version_msg {
            Some(version) => {
                println!("version: {:?}", version);
                output.node.state = db::NodeState::Online;
                db.insert(output.node);
            }
            None => {
                println!("version handshake failed");
                output.node.state = db::NodeState::Offline;
                db.insert(output.node);
            }
        }
        match output.addr_msg {
            Some(addr_msg) => {
                for net_addr in addr_msg {
                    let addr = net_addr.1.socket_addr().unwrap();
                    db.initialize(addr);
                }
            }
            None => println!("no addresses received"),
        }
    }
}

fn spawn(nthreads: i32, mut db: &'static mut db::NodeDb) {
    for _ in 0..nthreads {
        thread::spawn(move || {
            worker(&mut db.clone());
        });
    }
}

pub fn crawl() {
    let mut db = db::NodeDb::new();
    bootstrap(&mut db);
    spawn(100, &mut db);
    loop {
        thread::sleep(Duration::new(1 * 60, 0));
        let report = db.report();
        println!("{:?}", report);
    }
}
