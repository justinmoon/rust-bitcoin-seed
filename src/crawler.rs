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
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bitcoin::network::constants::Network;

// copied from murmel
const MAIN_SEEDER: [&str; 5] = [
    "seed.bitcoin.sipa.be",
    "dnsseed.bluematt.me",
    "dnsseed.bitcoin.dashjr.org",
    "seed.bitcoinstats.com",
    "seed.btc.petertodd.org",
];
const TEST_SEEDER: [&str; 4] = [
    "testnet-seed.bitcoin.jonasschnelli.ch",
    "seed.tbtc.petertodd.org",
    "seed.testnet.bitcoin.sprovoost.nl",
    "testnet-seed.bluematt.me",
];
pub fn dns_seed(network: Network) -> Vec<SocketAddr> {
    let mut seeds = Vec::new();
    if network == Network::Bitcoin {
        println!("reaching out for DNS seed...");
        for seedhost in MAIN_SEEDER.iter() {
            if let Ok(lookup) = (*seedhost, 8333).to_socket_addrs() {
                for host in lookup {
                    seeds.push(host);
                }
            } else {
                println!("{} did not answer", seedhost);
            }
        }
        println!("received {} DNS seeds", seeds.len());
    }
    if network == Network::Testnet {
        println!("reaching out for DNS seed...");
        for seedhost in TEST_SEEDER.iter() {
            if let Ok(lookup) = (*seedhost, 18333).to_socket_addrs() {
                for host in lookup {
                    seeds.push(host);
                }
            } else {
                println!("{} did not answer", seedhost);
            }
        }
        println!("received {} DNS seeds", seeds.len());
    }
    seeds
}

// copied from murmel
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

//struct Node {
//addr: SocketAddr,
//visits_missed: u32,
//last_missed_visit: u32,
//}

struct Result {
    socket_addr: SocketAddr,
    version_msg: Option<VersionMessage>,
    addr_msg: Option<Vec<(u32, Address)>>,
}

pub fn crawl() {
    let mut addrs: Vec<SocketAddr> = Vec::new();
    //let mut db: HashMap<SocketAddr, Node> = HashMap::new();
    addrs.extend(dns_seed(Network::Bitcoin));
    loop {
        let peer = addrs.pop().unwrap();
        let result = visit(peer);
        match result.version_msg {
            Some(version) => {
                println!("version: {:?}", version);
            }
            None => println!("version handshake failed"),
        }
        match result.addr_msg {
            Some(addr_msg) => {
                for addr in addr_msg {
                    addrs.push(addr.1.socket_addr().unwrap());
                }
            }
            None => println!("no addresses received"),
        }
        println!("Terminated with {} addrs", addrs.len());
    }
}

fn visit(peer: SocketAddr) -> Result {
    let mut result = Result {
        socket_addr: peer,
        version_msg: None,
        addr_msg: None,
    };
    println!("Connecting to {}", peer);
    match TcpStream::connect_timeout(&peer, Duration::new(1, 0)) {
        Ok(mut stream) => {
            println!("Connected");

            // timeout in 30 seconds
            stream
                .set_read_timeout(Some(Duration::new(30, 0)))
                .expect("Couldn't set timeout");

            // write version
            let lversion = compile_version();
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
                            result.version_msg = Some(rversion.clone());
                            let lverack = NetworkMessage::Verack;
                            stream
                                .write(&serialize(&RawNetworkMessage {
                                    magic: 0xd9b4bef9,
                                    payload: lverack,
                                }))
                                .expect("Couldn't write verack");
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
                                result.addr_msg = Some(addr.clone());
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
            return result;
        }
        Err(e) => {
            println!("Failed to connect: {}", e);
            return result;
        }
    }
}
