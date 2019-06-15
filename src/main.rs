use std::env;

mod crawler;
mod echo;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args[1] == "serve" {
        echo::serve();
    } else if args[1] == "client" {
        echo::client(args[2].as_bytes());
    } else if args[1] == "crawl" {
        crawler::crawl();
    }
}
