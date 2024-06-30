use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::thread;

#[derive(Clone)]
struct Args {
    directory: PathBuf,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            directory: PathBuf::from("./"),
        }
    }
}

fn parse_args() -> Args {
    let mut args = Args::default();
    let mut args_iter = std::env::args().skip(1);

    match args_iter.next().unwrap_or("".to_string()).as_str() {
        "--directory" => {
            args.directory = PathBuf::from(args_iter.next().unwrap());
            if !Path::exists(&args.directory) {
                std::fs::create_dir_all(&args.directory).unwrap();
            }
        }
        _ => {}
    }

    args
}

fn main() {
    let args = parse_args();

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        let cloned_args = args.clone();
        match stream {
            Ok(stream) => {
                thread::spawn(|| handle_client(stream, cloned_args));
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

#[derive(Default)]
enum RequestType {
    #[default]
    Get,
    Post,
}

#[derive(Default)]
struct HttpRequest {
    typ: RequestType,
    path: PathBuf,
    headers: HashMap<String, String>,
}

fn parse(http_request: &[String]) -> HttpRequest {
    let mut http_request_out: HttpRequest = Default::default();
    let mut req_it = http_request.iter();

    let mut it = req_it.next().unwrap().split_whitespace();

    if it.next().unwrap() == "POST" {
        http_request_out.typ = RequestType::Post;
    }

    http_request_out.path = PathBuf::from(it.next().unwrap());

    for line in req_it.by_ref() {
        dbg!(line);
        if line.trim().is_empty() {
            break;
        }
        if let Some((header, val)) = line.split_once(':') {
            http_request_out
                .headers
                .insert(header.to_string(), val.trim().to_string());
        } else {
            break;
        }
    }

    http_request_out
}

fn handle_post_request(
    http_request: HttpRequest,
    args: Args,
    mut stream_reader: BufReader<&mut TcpStream>,
) -> Option<String> {
    let path_components: Vec<_> = http_request.path.components().collect();
    match (
        path_components[1].as_os_str().to_str().unwrap(),
        http_request.headers["Content-Type"].as_str(),
    ) {
        ("files", "application/octet-stream") => {
            let mut buf = vec![
                0u8;
                http_request.headers["Content-Length"]
                    .parse::<usize>()
                    .unwrap()
            ];
            dbg!(&buf);
            stream_reader.read_exact(buf.as_mut()).unwrap();
            dbg!(&buf);
            std::fs::write(
                args.directory
                    .join(http_request.path.strip_prefix("/files/").unwrap()),
                buf,
            )
            .unwrap();
            Some("HTTP/1.1 201 Created\r\n\r\n".to_string())
        }
        _ => None,
    }
}

fn handle_get_request(
    http_request: HttpRequest,
    args: Args,
    mut _stream_reader: BufReader<&mut TcpStream>,
) -> Option<String> {
    if http_request.path == PathBuf::from("/") {
        return Some("HTTP/1.1 200 OK\r\n\r\n".to_string());
    }
    let path_components: Vec<_> = http_request.path.components().collect();
    match path_components[1].as_os_str().to_str().unwrap() {
        "echo" => Some(format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
            path_components[2].as_os_str().to_str().unwrap().len(),
            path_components[2].as_os_str().to_str().unwrap()
        )),
        "user-agent" => Some(format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
            http_request.headers["User-Agent"].len(),
            http_request.headers["User-Agent"]
        )),
        "files" => 
                std::fs::read_to_string(args.directory.join(http_request.path.strip_prefix("/files").unwrap())).ok().map(|f| {
                format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}",
            f.len(),
            f)}),
        _ => None,
    }
}

fn handle_client(mut stream: TcpStream, args: Args) {
    let mut stream_reader = BufReader::new(&mut stream);
    let mut http_request = vec![];
    loop {
        let mut line = String::new();
        stream_reader.read_line(&mut line).unwrap();
        if line.trim().is_empty() { break; }
        http_request.push(line.trim().to_string());
    }
    let http_request: HttpRequest = parse(&http_request);

    let response = match http_request.typ {
        RequestType::Get => handle_get_request(http_request, args, stream_reader),
        RequestType::Post => handle_post_request(http_request, args, stream_reader),
    };

    stream
        .write_all(
            response
                .unwrap_or_else(|| "HTTP/1.1 404 Not Found\r\n\r\n".to_string())
                .as_bytes(),
        )
        .unwrap();
}
