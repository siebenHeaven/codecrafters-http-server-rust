use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

#[derive(Clone)]
struct Args {
    directory: String,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            directory: "./".to_string(),
        }
    }
}

fn parse_args() -> Args {
    let mut args = Args::default();
    let mut args_iter = std::env::args().skip(1);

    match args_iter.next().unwrap_or("".to_string()).as_str() {
        "--directory" => {
            args.directory = args_iter.next().unwrap().to_string();
        }
        _ => {},
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

enum RequestType {
    Get,
    Post,
}

impl Default for RequestType {
    fn default() -> Self {
        RequestType::Get
    }
}

#[derive(Default)]
struct HttpRequest {
    typ: RequestType,
    path: String,
    headers: HashMap<String, String>,
    _body: Vec<String>,
}

fn parse(http_request: &Vec<String>) -> HttpRequest {
    let mut http_request_out: HttpRequest = Default::default();
    let mut req_it = http_request.iter();

    let mut it = req_it.next().unwrap().split_whitespace();

    if it.next().unwrap() == "POST" {
        http_request_out.typ = RequestType::Post;
    }

    http_request_out.path = it.next().unwrap().to_string();

    while let Some(Some((header, val))) = req_it.next().map(|line| line.split_once(':')) {
        http_request_out
            .headers
            .insert(header.to_string(), val.trim().to_string());
    }

    http_request_out
}

fn handle_get_request(http_request: HttpRequest, args: Args) -> String {
    if http_request.path == "/" {
        return "HTTP/1.1 200 OK\r\n\r\n".to_string();
    }
    let path_components: Vec<_> = http_request.path.split('/').collect();
    match path_components[1] {
        "echo" => format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
            path_components[2].len(),
            path_components[2]
        ),
        "user-agent" => format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
            http_request.headers["User-Agent"].len(),
            http_request.headers["User-Agent"]
        ),
        "files" => {
            if let Ok(f) =
                std::fs::read_to_string(format!("{}{}", args.directory, path_components[2]))
            {
                format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}",
            f.len(),
            f
        )
            } else {
                "HTTP/1.1 404 Not Found\r\n\r\n".to_string()
            }
        }
        _ => "HTTP/1.1 404 Not Found\r\n\r\n".to_string(),
    }
}

fn handle_client(mut stream: TcpStream, args: Args) {
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();
    println!("{}", http_request[0]);
    let http_request: HttpRequest = parse(&http_request);

    let response = match http_request.typ {
        RequestType::Get => handle_get_request(http_request, args),
        RequestType::Post => todo!(),
    };

    stream.write_all(response.as_bytes()).unwrap();
}
