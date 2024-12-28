use http::ThreadPool;
use std::{
    env, 
    net::TcpListener,
    path::PathBuf,
};

mod connection;

use connection::handle_connection;

fn main() {
    let listener: TcpListener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool: ThreadPool = ThreadPool::new(4);

    let default_root_path: PathBuf = PathBuf::from("pages");

    let args: Vec<String> = env::args().collect();
    let root_path: PathBuf;

    if args.len() > 2 {
        println!("Usage: http <path to pages folder>");
        return;
    } else if args.len() < 2 {
        root_path = default_root_path.clone();
    } else {
        root_path = PathBuf::from(&args[1]).join("html");
    }

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let root_path = root_path.clone();

        pool.execute(move || {
            handle_connection(stream, &root_path);
        });
    }
}