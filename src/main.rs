use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

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

        handle_connection(stream);
    }
}

/// Handle incoming TCP connections, print HTTP request and return the HTML response.
fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Request: {:#?}", http_request);

    let status_line = "HTTP/1.1 200 OK";
    let content_type = "Content-Type: text/html; charset=utf-8";
    let contents = fs::read_to_string("html/index.html").unwrap();
    let length = contents.len();

    let response = format!("{status_line}\r\n\
    {content_type}\r\n\
    Content-Length: {length}\r\n\r\n\
    {contents}");

    stream.write_all(response.as_bytes()).unwrap();
}
