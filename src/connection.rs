use std::{
    io::{prelude::*, BufReader},
    net::TcpStream,
    path::PathBuf,
};

use http::html::handle_html_request;
use http::static_files::handle_static_request;

pub fn handle_connection(mut stream: TcpStream, root_path: &PathBuf) {
    let buf_reader: BufReader<&TcpStream> = BufReader::new(&stream);
    let request_line: String = buf_reader.lines().next().unwrap().unwrap();

    let requested_path = request_line.split_whitespace().nth(1).unwrap_or("/");

    if requested_path.starts_with("/css/") || requested_path.starts_with("/js/") {
        handle_static_request(&mut stream, root_path, requested_path);
    } else {
        handle_html_request(&mut stream, root_path, requested_path);
    }
}
