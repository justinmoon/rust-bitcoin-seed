use bitcoin::consensus::encode::deserialize;
use bitcoin::consensus::encode::serialize;
use bitcoin::network::{
    address::Address, message::NetworkMessage, message::RawNetworkMessage,
    message_network::VersionMessage,
};

use byteorder::{ByteOrder, LittleEndian};
use std::io::{Read, Write};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream},
    time::{SystemTime, UNIX_EPOCH},
};

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

pub fn handshake(peer: String) {
    match TcpStream::connect(peer) {
        Ok(mut stream) => {
            println!("Connected");

            let msg = compile_version();
            let ser = serialize(&RawNetworkMessage {
                magic: 0xd9b4bef9,
                payload: msg,
            });

            stream.write(&ser).unwrap();
            println!("Sent message, awaiting reply...");

            // read magic
            let mut magic = vec![0u8; 4];
            stream.read_exact(&mut magic).unwrap();
            println!("Magic: {:?}", magic);

            // read command
            let mut command = vec![0u8; 12];
            stream.read_exact(&mut command).unwrap();
            println!("Command: {:?}", command);

            // read length
            let mut length = vec![0u8; 4];
            stream.read_exact(&mut length).unwrap();
            let length_int = LittleEndian::read_u32(&length);
            println!("Length: {:?}, Int: {}", length, length_int);

            // read checksum
            let mut checksum = vec![0u8; 4];
            stream.read_exact(&mut checksum).unwrap();
            println!("Checksum: {:?}", checksum);

            // read payload
            let mut payload = vec![0u8; length_int as usize];
            stream.read_exact(&mut payload).unwrap();
            println!("Payload: {:?}", payload);

            let ver: Result<VersionMessage, _> = deserialize(&payload);
            println!("Version: {:?}", ver);
        }
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
    println!("Terminated.");
}
