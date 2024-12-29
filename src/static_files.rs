use std::{fs, io::Write, net::TcpStream, path::PathBuf};

pub fn handle_static_request(stream: &mut TcpStream, root_path: &PathBuf, requested_path: &str) {
    let static_file_path = root_path.join(&requested_path[1..]);
    if static_file_path.exists() {
        let content = fs::read_to_string(&static_file_path).unwrap_or_default();
        let content_type = if requested_path.ends_with(".css") {
            "text/css"
        } else if requested_path.ends_with(".js") {
            "application/javascript"
        } else {
            "text/plain"
        };
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\n\r\n{}",
            content.len(),
            content
        );
        stream.write_all(response.as_bytes()).unwrap();
    } else {
        let error_message = "404 Not Found";
        let response = format!(
            "HTTP/1.1 404 NOT FOUND\r\nContent-Length: {}\r\n\r\n{}",
            error_message.len(),
            error_message
        );
        stream.write_all(response.as_bytes()).unwrap();
    }
}
