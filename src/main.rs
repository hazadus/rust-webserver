use std::net::TcpListener;

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
        println!("Connection established!");
    }
}
