use bitcoin::network::{
    address::Address, constants::Network, message::NetworkMessage, message_network::VersionMessage,
};
use env_logger;
use log::{info, trace, LevelFilter};
use std::error::Error;
use std::fmt;
use std::io;
use std::io::Write;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

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
        info!("reaching out for DNS seed...");
        for seedhost in MAIN_SEEDER.iter() {
            if let Ok(lookup) = (*seedhost, 8333).to_socket_addrs() {
                for host in lookup {
                    seeds.push(host);
                }
            } else {
                trace!("{} did not answer", seedhost);
            }
        }
        info!("received {} DNS seeds", seeds.len());
    }
    if network == Network::Testnet {
        info!("reaching out for DNS seed...");
        for seedhost in TEST_SEEDER.iter() {
            if let Ok(lookup) = (*seedhost, 18333).to_socket_addrs() {
                for host in lookup {
                    seeds.push(host);
                }
            } else {
                trace!("{} did not answer", seedhost);
            }
        }
        info!("received {} DNS seeds", seeds.len());
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

pub fn init_logger() {
    env_logger::Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {}",
                thread::current().name().unwrap(),
                record.level(),
                record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .init();
}

#[derive(Debug)]
pub struct CrawlerError {
    msg: String,
}

impl From<io::Error> for CrawlerError {
    fn from(error: io::Error) -> Self {
        CrawlerError {
            msg: error.to_string(),
        }
    }
}

impl From<bitcoin::consensus::encode::Error> for CrawlerError {
    fn from(error: bitcoin::consensus::encode::Error) -> Self {
        CrawlerError {
            msg: error.to_string(),
        }
    }
}
impl fmt::Display for CrawlerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Error for CrawlerError {}

impl CrawlerError {
    pub fn new(msg: String) -> CrawlerError {
        CrawlerError { msg }
    }
}
