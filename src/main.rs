use std::net::{TcpListener, TcpStream};
use std::io::Write;

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

fn handle_client(mut stream: TcpStream) {
    println!("Accepted\n");
    stream.write_all("HTTP/1.1 200 OK\r\n\r\n".as_bytes()).unwrap();
}
