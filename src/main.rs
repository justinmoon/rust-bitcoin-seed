use bitcoin::network::constants::Network;
use std::env;

mod crawler;
mod dns;
mod echo;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args[1] == "serve" {
        echo::serve();
    } else if args[1] == "client" {
        echo::client(args[2].as_bytes());
    } else if args[1] == "dns" {
        let seeds = dns::dns_seed(Network::Bitcoin);
        println!("{:?}", seeds);
    } else if args[1] == "crawl" {
        crawler::crawl();
    }
}
