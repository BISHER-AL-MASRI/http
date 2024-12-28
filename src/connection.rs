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

    let requested_path = request_line.split_whitespace().nth(1).unwrap_or("/");

    if requested_path.starts_with("/css/") || requested_path.starts_with("/js/") {
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
            return;
        }
    }

    let cssfiles = get_files(root_path)
        .into_iter()
        .filter(|file| file.extension() == Some(std::ffi::OsStr::new("css")))
        .collect::<Vec<_>>();

    let javascriptfiles = get_files(root_path)
        .into_iter()
        .filter(|file| file.extension() == Some(std::ffi::OsStr::new("js")))
        .collect::<Vec<_>>();

    let global_css = cssfiles
        .iter()
        .find(|file| file.file_name().unwrap().to_string_lossy() == "styles.css");

    let mut css_links = String::new();
    if let Some(file) = global_css {
        if let Some(filename) = file.file_name() {
            let css_path = format!("css/{}", filename.to_string_lossy());
            let link_tag = format!("<link rel=\"stylesheet\" href=\"/{}\">", css_path);
            css_links.push_str(&link_tag);
        }
    }

    let mut javascript_links = String::new();

    let htmlfiles = get_files(root_path);
    println!("Found pages: {:?}", htmlfiles);

    let index_file = htmlfiles.iter().find(|file| file.ends_with("index.html"));
    let not_found_file = htmlfiles.iter().find(|file| file.ends_with("404.html"));

    let (status_line, filename): (&str, PathBuf) = if requested_path == "/" {
        if let Some(file) = index_file {
            ("HTTP/1.1 200 OK", file.clone())
        } else {
            (
                "HTTP/1.1 404 NOT FOUND",
                not_found_file.unwrap_or(&PathBuf::from("404.html")).clone(),
            )
        }
    } else {
        let sanitized_path = requested_path.trim_start_matches('/');
        let target_file = htmlfiles.iter().find(|file| {
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

    let specific_css = cssfiles.iter().find(|file| {
        let css_name = file.file_stem().map(|s| s.to_string_lossy().to_string());
        filename
            .file_stem()
            .map(|name| name.to_string_lossy().to_string() == css_name.unwrap_or_default())
            .unwrap_or(false)
    });

    if let Some(file) = specific_css {
        if let Some(filename) = file.file_name() {
            let css_path = format!("css/{}", filename.to_string_lossy());
            let link_tag = format!("<link rel=\"stylesheet\" href=\"/{}\">", css_path);
            css_links.push_str(&link_tag);
        }
    }

    let specific_javascript = javascriptfiles.iter().find(|file| {
        let javascript_name = file.file_stem().map(|s| s.to_string_lossy().to_string());
        filename
            .file_stem()
            .map(|name| name.to_string_lossy().to_string() == javascript_name.unwrap_or_default())
            .unwrap_or(false)
    });

    if let Some(file) = specific_javascript {
        if let Some(filename) = file.file_name() {
            let javascript_path = format!("js/{}", filename.to_string_lossy());
            let script_tag = format!("<script src=\"/{}\"></script>", javascript_path);
            javascript_links.push_str(&script_tag);
        }
    }

    let response: String = match fs::read_to_string(&filename) {
        Ok(mut contents) => {
            if let Some(pos) = contents.find("</head>") {
                contents.insert_str(pos, &css_links);
                contents.insert_str(pos + css_links.len(), &javascript_links);
            } else {
                contents.insert_str(
                    0,
                    &format!("<head>{}\n{}</head>", css_links, javascript_links),
                );
            }

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
