use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_client(stream);
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

struct HttpRequest<'a> {
    typ: RequestType,
    path: &'a str,
}

impl Default for HttpRequest<'_> {
    fn default() -> Self {
        Self {
            typ: RequestType::Get,
            path: Default::default(),
        }
    }
}

fn parse(http_request: &Vec<String>) -> HttpRequest {
    let mut http_request_out: HttpRequest = Default::default();

    let mut it = http_request[0].split_whitespace();

    if it.next().unwrap() == "POST" {
        http_request_out.typ = RequestType::Post;
    }

    http_request_out.path = it.next().unwrap();

    http_request_out
}

fn handle_get_request(http_request: HttpRequest) -> String {
    if http_request.path == "/" {
        return "HTTP/1.1 200 OK\r\n\r\n".to_string();
    }
    let path_components: Vec<_> = http_request.path.split('/').collect();
    match path_components[1] {
        "echo" => {
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                path_components[2].len(),
                path_components[2]
            )
        }
        _ => "HTTP/1.1 404 Not Found\r\n\r\n".to_string(),
    }
}

fn handle_client(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();
    println!("{}", http_request[0]);
    let http_request: HttpRequest = parse(&http_request);

    let response = match http_request.typ {
        RequestType::Get => handle_get_request(http_request),
        RequestType::Post => todo!(),
    };

    stream.write_all(response.as_bytes()).unwrap();
}
