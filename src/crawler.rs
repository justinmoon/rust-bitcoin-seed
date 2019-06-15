use bitcoin::consensus::encode::{deserialize, serialize};
use bitcoin::network::{
    address::Address,
    message::{NetworkMessage, RawNetworkMessage},
    message_network::VersionMessage,
    stream_reader::StreamReader,
};
use byteorder::{ByteOrder, LittleEndian};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn compile_version() -> NetworkMessage {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8333);

    let dummy_addr = Address::new(&addr, 0);
    NetworkMessage::Version(VersionMessage {
        version: 70015,
        services: 0,
        timestamp,
        receiver: dummy_addr.clone(),
        sender: dummy_addr.clone(),
        nonce: 0,
        user_agent: String::from("/Satoshi:0.17.1/"),
        start_height: 1,
        relay: false,
    })
}

struct Node {
    ip: IpAddr,
    port: u32,
    visits_missed: u32,
    last_missed_visit: u32,
}

struct Result {
    node: Node,
    version: NetworkMessage,
    addrs: NetworkMessage,
}

pub fn crawl() {
    let mut addrs: Vec<Address> = Vec::new();
    //let a1 = visit("194.71.109.46:8333".to_string());
    //for record in a1 {
    //addrs.push(record.1.clone());
    //}
    //println!("Terminated with {} addrs", addrs.len());
    let a2 = visit("91.106.188.229:8333".to_string());
    for record in a2 {
        addrs.push(record.1.clone());
    }
    println!("Terminated with {} addrs", addrs.len());
    println!("starting loop");
    while true {
        let next = addrs.pop().unwrap();
        let ip = next.address;
        let port = next.port;
        let peer = format!(
            "{}.{}.{}.{}:{}",
            ip[6] / 256,
            ip[6] % 256,
            ip[7] / 256,
            ip[7] % 256,
            port
        );
        let addr = visit(peer);
        for record in addr {
            addrs.push(record.1.clone());
        }
        println!("Terminated with {} addrs", addrs.len());
    }
}

fn visit(peer: String) -> std::vec::Vec<(u32, bitcoin::network::address::Address)> {
    println!("Connecting to {}", peer);
    match TcpStream::connect_timeout(&peer.parse().unwrap(), Duration::new(10, 0)) {
        Ok(mut stream) => {
            println!("Connected");

            // write version
            let lversion = compile_version();
            stream
                .write(&serialize(&RawNetworkMessage {
                    magic: 0xd9b4bef9,
                    payload: lversion,
                }))
                .unwrap();
            println!("Sent message, awaiting reply...");

            // read version
            let mut reader = StreamReader::new(&mut stream, Some(100000));
            let rversion = reader.next_message().unwrap();
            println!("{:?}", rversion);

            // read verack
            let rverack = reader.next_message().unwrap();
            println!("{:?}", rverack);

            // write verack
            let lverack = NetworkMessage::Verack;
            stream.write(&serialize(&RawNetworkMessage {
                magic: 0xd9b4bef9,
                payload: lverack,
            }));

            // request peer's peers
            let getaddr = NetworkMessage::GetAddr;
            stream.write(&serialize(&RawNetworkMessage {
                magic: 0xd9b4bef9,
                payload: getaddr,
            }));

            // event loop prints messages as they arrive
            while true {
                let mut reader = StreamReader::new(&mut stream, Some(10000000));
                match reader.next_message() {
                    Ok(message) => match message.payload {
                        NetworkMessage::Addr(ref addr) => {
                            println!("Received {} addrs", addr.len());
                            return addr.clone();
                        }
                        _ => {
                            println!("Received {}", message.command());
                        }
                    },
                    Err(err) => {
                        println!("Error: {}", err);
                    }
                }
            }
            return vec![];
        }
        Err(e) => {
            println!("Failed to connect: {}", e);
            return vec![];
        }
    }
}

//pub fn crawl() {
//let mut nodes: HashMap<IpAddr, Node> = HashMap::new();
//// channel for version messages
//// channel for addrs
//// or 1 channel that can accept structs ...
//}
