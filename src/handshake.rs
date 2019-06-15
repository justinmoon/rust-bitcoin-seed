use bitcoin::consensus::encode::{deserialize, serialize};
use bitcoin::network::{
    address::Address,
    message::{NetworkMessage, RawNetworkMessage},
    message_network::VersionMessage,
    stream_reader::StreamReader,
};
use byteorder::{ByteOrder, LittleEndian};
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::time::{SystemTime, UNIX_EPOCH};

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

pub fn _handshake_by_hand(peer: String) {
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

pub fn handshake(peer: String) {
    match TcpStream::connect(peer) {
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
                    Ok(message) => println!("Mesage: {:?}", message),
                    Err(err) => println!("Error: {}", err),
                }
            }
        }
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
    println!("Terminated.");
}
