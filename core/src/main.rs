use std::{io::{BufRead, Bytes, Cursor, Read, Write}, net::{SocketAddr, TcpStream}, sync::mpsc::{Receiver, Sender}, thread::JoinHandle};

use binrw::{io::BufReader, BinRead};
use binrw::BinReaderExt;
use protocol::Message;


mod protocol;

fn handle_connection(stream: TcpStream, addr: SocketAddr) {
    println!("Handling connection: {:#?}", addr);
}

fn main() {
    let (dispatch_in_send, dispatch_in_recv): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = std::sync::mpsc::channel(); 
    let (dispatch_out_send, dispatch_out_recv): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = std::sync::mpsc::channel(); 

    let dispatch_thread = std::thread::spawn(move || {
        let receiver = dispatch_in_recv;
        let sender = dispatch_out_send;

        loop {
            match receiver.recv() {
                Ok(bytes) => {
                    let response = format!("Recieved {} bytes", bytes.len());
                    println!("{}", response);
                    match sender.send(response.as_bytes().to_vec()) {
                        Ok(_) => println!("Response sent successfully"),
                        Err(err) => println!("Failed to send response: {err}"),
                    }
                },
                Err(err) => println!("Recv failed: {err}"),
            }
    
            // dispatch message, receive response

        }
    });

    let listener = match  std::net::TcpListener::bind("0.0.0.0:4444") {
        Ok(listener) => listener,
        Err(err) => panic!("Failed to start listener: {err}"),
    };


    loop {

        let (mut stream, addr) = match listener.accept() {
            Ok((stream, addr)) => (stream, addr),
            Err(err) => {
                println!("Failed to accept incoming connectiong: {err}");
                break;
            }
        };

        println!("Received connection from: {:#?}", addr);

        let mut reader = binrw::io::NoSeek::new(&stream);
        match protocol::Message::read(&mut reader) {
            Ok(Message::Load { len, message }) => {
                println!("Got load message:\n\tlen {len}\n\t{}", message.len());
            },
            Err(err) => {
                println!("Failed to read from stream: {err}");
                continue;
            }
        };



        match dispatch_out_recv.recv() {
            Ok(mut response) => {
                match stream.write_all(response.as_mut_slice()) {
                    Ok(_) => println!("Wrote {} bytes to stream", response.len()),
                    Err(err) => println!("Failed to write bytes to stream: {err}"),
                }
                println!("Received response from dispatch: {}", String::from_utf8(response).unwrap());
                
            },
            Err(err) => println!("Failed to receive response from dispatch: {err}")
        }
    }

}
