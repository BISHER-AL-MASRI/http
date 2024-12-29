use std::{fs, io::Write, net::TcpStream, path::PathBuf};

use crate::readfolder::get_files;

pub fn handle_html_request(stream: &mut TcpStream, root_path: &PathBuf, requested_path: &str) {
    let htmlfiles = get_files(root_path);
    let cssfiles = filter_files_by_extension(root_path, "css");
    let javascriptfiles = filter_files_by_extension(root_path, "js");

    let (status_line, filename) = resolve_requested_file(root_path, requested_path, &htmlfiles);

    let response = match fs::read_to_string(&filename) {
        Ok(mut contents) => {
            let css_links = generate_global_links(&cssfiles, "css", "stylesheet");
            let js_links =
                generate_directory_matched_links(&javascriptfiles, &filename, "js", "script");
            let page_specific_css_links =
                generate_directory_matched_links(&cssfiles, &filename, "css", "stylesheet");

            let full_css_links = format!("{}{}", css_links, page_specific_css_links);

            if let Some(pos) = contents.find("</head>") {
                contents.insert_str(pos, &full_css_links);
                contents.insert_str(pos + full_css_links.len(), &js_links);
            } else {
                contents.insert_str(0, &format!("<head>{}\n{}</head>", full_css_links, js_links));
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

fn resolve_requested_file(
    root_path: &PathBuf,
    requested_path: &str,
    htmlfiles: &[PathBuf],
) -> (String, PathBuf) {
    let sanitized_path = requested_path.trim_start_matches('/').replace("/", "/");
    let requested_file = root_path.join("html").join(&sanitized_path);

    if requested_file.is_dir() {
        let index_file = requested_file.join("index.html");
        if index_file.exists() {
            return ("HTTP/1.1 200 OK".to_owned(), index_file);
        }
    } else if requested_file.exists() {
        return ("HTTP/1.1 200 OK".to_owned(), requested_file);
    }

    if !requested_path.contains('.') {
        let html_file = root_path
            .join("html")
            .join(format!("{}.html", sanitized_path));
        if html_file.exists() {
            return ("HTTP/1.1 200 OK".to_owned(), html_file);
        }
    }

    let not_found_file = htmlfiles.iter().find(|file| file.ends_with("404.html"));
    if let Some(file) = not_found_file {
        ("HTTP/1.1 404 NOT FOUND".to_owned(), file.clone())
    } else {
        (
            "HTTP/1.1 404 NOT FOUND".to_owned(),
            root_path.join("html").join("404.html"),
        )
    }
}

fn filter_files_by_extension(root_path: &PathBuf, ext: &str) -> Vec<PathBuf> {
    get_files(root_path)
        .into_iter()
        .filter(|file| file.extension() == Some(std::ffi::OsStr::new(ext)))
        .collect::<Vec<_>>()
}

fn get_relative_directory(path: &PathBuf, base_folder: &str) -> Option<PathBuf> {
    path.strip_prefix(path.ancestors().find(|p| {
        p.file_name()
            .map(|name| name.to_string_lossy() == base_folder)
            .unwrap_or(false)
    })?)
    .ok()
    .map(|p| p.to_path_buf())
}

fn generate_directory_matched_links(
    files: &[PathBuf],
    html_file: &PathBuf,
    folder: &str,
    tag_type: &str,
) -> String {
    let mut links = String::new();

    if let Some(html_rel_path) = get_relative_directory(html_file, "html") {
        let html_parent = html_rel_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new(""));
        let html_stem = html_file.file_stem().unwrap_or_default();

        for file in files {
            if let Some(file_rel_path) = get_relative_directory(file, folder) {
                let file_parent = file_rel_path
                    .parent()
                    .unwrap_or_else(|| std::path::Path::new(""));

                if file.file_stem() == Some(html_stem) && file_parent == html_parent {
                    if let Some(filename) = file.file_name() {
                        let path = if file_parent.as_os_str().is_empty() {
                            format!("{}/{}", folder, filename.to_string_lossy())
                        } else {
                            format!(
                                "{}/{}/{}",
                                folder,
                                file_parent.display(),
                                filename.to_string_lossy()
                            )
                        };

                        if tag_type == "stylesheet" {
                            links
                                .push_str(&format!("<link rel=\"stylesheet\" href=\"/{}\">", path));
                        } else if tag_type == "script" {
                            links.push_str(&format!("<script src=\"/{}\"></script>", path));
                        }
                    }
                }
            }
        }
    }

    links
}

fn generate_global_links(files: &[PathBuf], folder: &str, tag_type: &str) -> String {
    let mut links = String::new();

    for file in files {
        if let Some(filename) = file.file_name() {
            if filename == "styles.css"
                && file
                    .parent()
                    .map_or(true, |p| p.file_name().map_or(true, |name| name == folder))
            {
                let path = format!("{}/{}", folder, filename.to_string_lossy());
                if tag_type == "stylesheet" {
                    links.push_str(&format!("<link rel=\"stylesheet\" href=\"/{}\">", path));
                }
            }
        }
    }

    links
}
