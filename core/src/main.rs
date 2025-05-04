use std::{any::{type_name, TypeId}, env, io::{BufRead, Bytes, Cursor, Read, Write}, net::{SocketAddr, TcpStream}, os::raw::c_void, sync::mpsc::{Receiver, Sender}, thread::JoinHandle};

use binrw::{io::BufReader, BinRead, BinWrite};
use binrw::BinReaderExt;
use memory_module::{allocator::VirtualAlloc, FnDispatch, MemoryModule};

fn handle_connection(stream: TcpStream, addr: SocketAddr) {
    println!("Handling connection: {:#?}", addr);
}

include!("codegen/protos/protocol.rs");

use prost::{self, Message};

use binrw;
use rmp_serde::encode::write;

#[binrw::binrw]
#[brw(big)]
struct Envelope {
    #[br(temp)]
    #[bw(calc = data.len() as u64)]
    pub len: u64,

    #[br(count = len)]
    pub data: Vec<u8>,
}

fn main() {
    let (dispatch_in_send, dispatch_in_recv): (Sender<protocol::Msg>, Receiver<protocol::Msg>) = std::sync::mpsc::channel(); 
    let (dispatch_out_send, dispatch_out_recv): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = std::sync::mpsc::channel(); 

    let dispatch_thread = std::thread::spawn(move || {
        let receiver = dispatch_in_recv;
        let sender = dispatch_out_send;

        let mut modules: Vec<(MemoryModule<VirtualAlloc>, FnDispatch)> = Vec::new();

        loop {
            let response = match receiver.recv() {
                Ok(protocol::Msg::Load(load)) => {
                    println!("Got load message:\n\t{}", load.data.len());

                    println!("initializing memory module");
                    let mut module = MemoryModule::<VirtualAlloc>::new(load.data);

                    println!("callling module.load_library");
                    match module.load_library() {
                        Ok(true) => {
                            println!("successfully loaded module");
                            match module.get_proc_address("dispatch") {
                                Some(dispatch) => {
                                    let dispatch: FnDispatch = unsafe { std::mem::transmute(dispatch) };
                                    modules.push((module, dispatch));
                                    let module_id: u32 = modules.len() as u32 - 1;
                                    let msg = protocol::Msg::LoadResponse(LoadResponse { module_id });
                                    let mut response: Vec<u8> = Vec::new();
                                    msg.encode(&mut response);

                                    let envelope = Envelope{
                                        data: response
                                    };
                                    let writer_buf: Vec<u8> = Vec::new();
                                    let mut cursor = Cursor::new(writer_buf);
                                    envelope.write(&mut cursor).unwrap();
                                    cursor.into_inner()
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
                Ok(protocol::Msg::Command(mut command)) => {
                    println!("Got command message:\n\tid: {}\n\targs: {}", command.id, command.data.len());

                    match modules.get(command.module_id as usize) {
                        Some((_, dispatch)) => {
                            let mut response_ptr: *mut u8 = std::ptr::null_mut();
                            let mut response_len = 0usize;
                            println!("calling dispatch");
                            unsafe { dispatch(command.id as usize, command.data.as_mut_ptr(), command.data.len(), &mut response_ptr, &mut response_len); }
                            let response = unsafe { Vec::from_raw_parts(response_ptr, response_len, response_len) };

                            let msg = protocol::Msg::Response(Response { data: response });
                            let mut response: Vec<u8> = Vec::new();
                            msg.encode(&mut response);
                            response

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

        let envelope = Envelope::read(&mut reader).unwrap();

        let msg = Protocol::decode(envelope.data.as_slice()).unwrap();
        let msg = msg.msg.unwrap();
       

        match dispatch_in_send.send(msg) {
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
