use std::{any::{type_name, TypeId}, io::{BufRead, Bytes, Cursor, Read, Write}, net::{SocketAddr, TcpStream}, os::raw::c_void, sync::mpsc::{Receiver, Sender}, thread::JoinHandle};

use binrw::{io::BufReader, BinRead, BinWrite};
use binrw::BinReaderExt;
use memory_module::{allocator::VirtualAlloc, FnDispatch, MemoryModule};
use protocol::{Header, Message};


mod protocol;

fn handle_connection(stream: TcpStream, addr: SocketAddr) {
    println!("Handling connection: {:#?}", addr);
}


fn main() {
    let (dispatch_in_send, dispatch_in_recv): (Sender<Message>, Receiver<Message>) = std::sync::mpsc::channel(); 
    let (dispatch_out_send, dispatch_out_recv): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = std::sync::mpsc::channel(); 

    let dispatch_thread = std::thread::spawn(move || {
        let receiver = dispatch_in_recv;
        let sender = dispatch_out_send;

        let mut modules: Vec<(MemoryModule<VirtualAlloc>, FnDispatch)> = Vec::new();

        loop {
            let response = match receiver.recv() {
                Ok(Message::Load(message)) => {
                    println!("Got load message:\n\t{}", message.data.len());

                    println!("initializing memory module");
                    let mut module = MemoryModule::<VirtualAlloc>::new(message.data);

                    println!("callling module.load_library");
                    match module.load_library() {
                        Ok(true) => {
                            println!("successfully loaded module");
                            match module.get_proc_address("dispatch") {
                                Some(dispatch) => {
                                    let dispatch: FnDispatch = unsafe { std::mem::transmute(dispatch) };
                                    modules.push((module, dispatch));
                                    let module_id: u32 = modules.len() as u32 - 1;
                                    let mut response = Cursor::new(Vec::<u8>::new());
                                    let message_data = module_id.to_be_bytes().to_vec();
                                    let message = protocol::Response{data: message_data};
                                    match message.write(&mut response) {
                                        Ok(_) => response.into_inner(),
                                        Err(err) => {
                                            println!("Failed to serialize response: {err}");
                                            Vec::new()
                                        }
                                    }
                                },
                                None => Vec::new() 
                            }
                        },
                        Ok(false) | Err(_) => {
                            println!("Failed to load module");
                            Vec::new()
                        }
                    }

                },
                Ok(Message::Command(mut command)) => {
                    println!("Got command message:\n\tid: {}\n\targs: {}", command.id, command.data.len());

                    match modules.get(command.module_id as usize) {
                        Some((_, dispatch)) => {
                            let mut response_ptr: *mut u8 = std::ptr::null_mut();
                            let mut response_len = 0usize;
                            println!("calling dispatch");
                            unsafe { dispatch(command.id as usize, command.data.as_mut_ptr(), command.data.len(), &mut response_ptr, &mut response_len); }
                            let response = unsafe { Vec::from_raw_parts(response_ptr, response_len, response_len) };

                            let mut writer = Cursor::new(Vec::<u8>::new());
                            let message = protocol::Response { data: response };
                            match message.write(&mut writer) {
                                Ok(_) => writer.into_inner(),
                                Err(err) => {
                                    println!("Failed to serialize response: {err}");
                                    Vec::new()
                                } 
                            }
                        },
                        None => Vec::new()
                    }
                },
                Err(err) => {
                    println!("Recv failed: {err}");
                    Vec::new()
                }
                _ => {
                    println!("Uexpected message type");
                    Vec::new()
                }
            };
    
            // dispatch message, receive response
            let response_len = response.len();
            match sender.send(response) {
                Ok(_) => println!("Sent response of {} bytes", response_len),
                Err(err) => println!("Failed to send response: {err}"),
            }

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

        println!("Reading header");
        let header = Header::read(&mut reader).unwrap();
        println!("Read header: {header:#?}");

        let message = match header.r#type {
            protocol::Type::Load => {
                println!("trying to read load message");
                let message = protocol::Load::read(&mut reader).unwrap();
                println!("got message: {}", message.data.len());
                protocol::Message::Load(message)
            },
            protocol::Type::Command => {
                println!("trying to read command message");
                let message = protocol::Command::read(&mut reader).unwrap();
                println!("got message: {}", message.data.len());
                protocol::Message::Command(message)
            }
            _ => {
                println!("Got command: {:#?}", header.r#type);
                continue;
            }
        };

        match dispatch_in_send.send(message) {
            Ok(_) => println!("sent message to dispatch thread"),
            Err(err) => println!("Failed to send to dispatch thread: {err}")
        }

        match dispatch_out_recv.recv() {
            Ok(mut response) => {
                match stream.write_all(response.as_mut_slice()) {
                    Ok(_) => println!("Wrote {} bytes to stream", response.len()),
                    Err(err) => println!("Failed to write bytes to stream: {err}"),
                }                
            },
            Err(err) => println!("Failed to receive response from dispatch: {err}")
        }
    }

}
