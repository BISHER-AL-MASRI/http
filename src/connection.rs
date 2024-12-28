use std::{
    fs,
    io::{prelude::*, BufReader},
    net::TcpStream,
    path::PathBuf,
};

use http::readfolder::get_files;

pub fn handle_connection(mut stream: TcpStream, root_path: &PathBuf) {
    let buf_reader: BufReader<&TcpStream> = BufReader::new(&stream);
    let request_line: String = buf_reader.lines().next().unwrap().unwrap();

    let files = get_files(root_path);
    println!("Found pages: {:?}", files);

    let index_file = files.iter().find(|file| file.ends_with("index.html"));
    let not_found_file = files.iter().find(|file| file.ends_with("404.html"));

    let requested_path = request_line
        .split_whitespace()
        .nth(1)
        .unwrap_or("/");

    // Determine the file to serve
    let (status_line, filename): (&str, PathBuf) = if requested_path == "/" {
        if let Some(file) = index_file {
            ("HTTP/1.1 200 OK", file.clone())
        } else {
            ("HTTP/1.1 404 NOT FOUND", not_found_file.unwrap_or(&PathBuf::from("404.html")).clone())
        }
    } else {
        let sanitized_path = requested_path.trim_start_matches('/');
        let target_file = files.iter().find(|file| {
            file.file_name()
                .map(|name| name.to_string_lossy() == format!("{sanitized_path}.html"))
                .unwrap_or(false)
        });

        if let Some(file) = target_file {
            ("HTTP/1.1 200 OK", file.clone())
        } else if let Some(file) = not_found_file {
            ("HTTP/1.1 404 NOT FOUND", file.clone())
        } else {
            ("HTTP/1.1 404 NOT FOUND", PathBuf::from("404.html"))
        }
    };

    let response: String = match fs::read_to_string(&filename) {
        Ok(contents) => {
            let length = contents.len();
            format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}")
        }
        Err(_) => {
            let error_message = "500 Internal Server Error";
            format!(
                "HTTP/1.1 500 INTERNAL SERVER ERROR\r\nContent-Length: {}\r\n\r\n{}",
                error_message.len(),
                error_message
            )
        }
    };

    stream.write_all(response.as_bytes()).unwrap();
}
