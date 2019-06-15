use bitcoin::consensus::encode::{deserialize, serialize};
use bitcoin::network::{
    address::Address,
    message::{NetworkMessage, RawNetworkMessage},
    message_network::VersionMessage,
    stream_reader::StreamReader,
};
use byteorder::{ByteOrder, LittleEndian};
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::tcp::TcpStream;
use tokio::prelude::*;

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

//pub fn handshake(peer: String) {
//let addr = peer.parse().unwrap();
//let fut = TcpStream::connect(&addr).and_then(|stream| {
//println!("Connected");

//// write version
//let lversion = compile_version();
//let msg = &serialize(&RawNetworkMessage {
//magic: 0xd9b4bef9,
//payload: lversion,
//});
//tokio::io::write_all(stream, msg)
//});
////.and_then(|stream| {
////// read version
////let mut reader = StreamReader::new(&mut stream, Some(100000));
////let rversion = reader.next_message().unwrap()
//////println!("Received version: {:?}", rversion);
////})
////.and_then(|stream| {
////// read verack
////let rverack = stream.next_message().unwrap()
//////println!("Received verack: {:?}", rverack);
////})
////.and_then(|stream| {
////// write verack
////let lverack = NetworkMessage::Verack;
////stream.write(&serialize(&RawNetworkMessage {
////magic: 0xd9b4bef9,
////payload: lverack,
////}))
////})
////.and_then(|stream| {
//////event loop prints messages as they arrive
////while true {
////let message = stream.next_message().unwrap();
////println!("{:?}", message);
////}
////});
//}

pub fn handshake(peer: String) {
    let addr = peer.parse().unwrap();
    let fut = TcpStream::connect(&addr).and_then(|stream| {
        println!("Connected");

        let (_reader, writer) = stream.split();
        // write version
        let lversion = compile_version();
        let msg = &serialize(&RawNetworkMessage {
            magic: 0xd9b4bef9,
            payload: lversion,
        });
        tokio::io::write_all(writer, msg)
    });
    //.and_then(|stream| {
    //// read version
    //let mut reader = StreamReader::new(&mut stream, Some(100000));
    //let rversion = reader.next_message().unwrap()
    ////println!("Received version: {:?}", rversion);
    //})
    //.and_then(|stream| {
    //// read verack
    //let rverack = stream.next_message().unwrap()
    ////println!("Received verack: {:?}", rverack);
    //})
    //.and_then(|stream| {
    //// write verack
    //let lverack = NetworkMessage::Verack;
    //stream.write(&serialize(&RawNetworkMessage {
    //magic: 0xd9b4bef9,
    //payload: lverack,
    //}))
    //})
    //.and_then(|stream| {
    ////event loop prints messages as they arrive
    //while true {
    //let message = stream.next_message().unwrap();
    //println!("{:?}", message);
    //}
    //});
}
