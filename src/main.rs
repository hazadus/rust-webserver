//! # rust-webserver
//! Simple webserver built to learn basics of Rust backend development.
use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};
use rust_webserver::ThreadPool;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(5);

    /*
    A single stream represents an open connection between the client and the server.
    A connection is the name for the full request and response process in which a client connects
    to the server, the server generates a response, and the server closes the connection. As such,
    we will read from the TcpStream to see what the client sent and then write our response to the
    stream to send data back to the client. Overall, this for loop will process each connection in
    turn and produce a series of streams for us to handle.
    */
    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }

    println!("Shutting down.");
}

/// Handle incoming TCP connections, print HTTP request and return the HTML response.
fn handle_connection(mut stream: TcpStream) {
    // Read HTTP request and print it
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    // Print request and host
    println!("Request: {:#?} from {:#?}", http_request[0], http_request[1]);

    // Check the first line of the request and respond with HTML
    let contents: String;
    let content_type = "Content-Type: text/html; charset=utf-8";
    let request = String::from(&http_request[0]);
    let request_tokens: Vec<&str> = request.split(" ").collect();
    let request_method = request_tokens[0];
    let requested_file = format!("html{}", request_tokens[1]);
    let mut file_path = "html/404.html";
    let mut status_code = "HTTP/1.1 404 NOT FOUND";

    // We need to explicitly match on a slice of request_line to pattern match against the string
    // literal values; match does not do automatic referencing and dereferencing like the equality
    // method does. Hence `[..]` in `match &http_request[0][..]`.
    let (status_line, file_path) = match &http_request[0][..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "html/index.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "html/index.html")
        },
        _ => {
            if request_method == "GET" {
                let file_exists = std::path::Path::new(&requested_file).exists();

                if file_exists {
                    status_code = "HTTP/1.1 200 OK";
                    file_path = &requested_file;
                } else {
                    println!("File does not exist: {}", requested_file);
                }
            }
            (status_code, file_path)
        }
    };

    // NB: number of `\r\n`'s matters!
    contents = fs::read_to_string(file_path).unwrap();
    let length = contents.len();
    let response = format!("{status_line}\r\n\
            {content_type}\r\n\
            Content-Length: {length}\r\n\r\n\
            {contents}");

    stream.write_all(response.as_bytes()).unwrap();
}
