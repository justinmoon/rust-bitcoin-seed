use std::io;
use std::net::{TcpListener, TcpStream};

fn handle_client(stream: TcpStream) {
    println!("connection established");
    //let data = stream.read_to_string()?;
    //println!("received: {}", data);
}

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7777")?;

    // accept connections and process them serially
    for stream in listener.incoming() {
        handle_client(stream?);
    }
    Ok(())
}
