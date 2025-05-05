use binrw::BinWrite;
use memory_module::{allocator::VirtualAlloc, FnDispatch, MemoryModule};
use std::{io, sync::mpsc};
use std::thread;

include!("codegen/protos/protocol.rs");

use binrw;
use server::Server;

mod server;
mod remote;
mod protocolext;

#[binrw::binrw]
#[brw(big)]
struct Envelope {
    #[br(temp)]
    #[bw(calc = data.len() as u64)]
    pub len: u64,

    #[br(count = len)]
    pub data: Vec<u8>,
}

impl Envelope {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data
        }
    }
}

fn main() {
    let (dispatch_in_send, dispatch_in_recv)= mpsc::channel::<(protocol::Msg, mpsc::Sender::<Vec<u8>>)>(); 

    let mut threads: Vec<thread::JoinHandle<()>> = Vec::new();

    let dispatch_thread = std::thread::spawn(move || {
        let receiver = dispatch_in_recv;

        let mut modules: Vec<(MemoryModule<VirtualAlloc>, FnDispatch)> = Vec::new();

        loop {

            match receiver.recv() {
                Ok((protocol::Msg::Load(load), sender)) => {
                    println!("Got load message:\n\t{}", load.data.len());

                    println!("initializing memory module");
                    let mut module = MemoryModule::<VirtualAlloc>::new(load.data);

                    println!("callling module.load_library");
                    match module.load_library() {
                        Ok(true) => {
                            println!("successfully loaded module");
                            if let Some(dispatch) =  module.get_proc_address("dispatch") {
                                let dispatch: FnDispatch = unsafe { std::mem::transmute(dispatch) };
                                modules.push((module, dispatch));
                                let module_id: u32 = modules.len() as u32 - 1;
                                let msg = protocol::Msg::LoadResponse(LoadResponse { module_id });
                                let mut response: Vec<u8> = Vec::new();
                                msg.encode(&mut response);

                                let response_len = response.len();
                                match sender.send(response) {
                                    Ok(_) => println!("Sent response of {} bytes", response_len),
                                    Err(err) => println!("Failed to send response: {err}"),
                                }                            }
                        },
                        Ok(false) | Err(_) => {
                            println!("Failed to load module");
                        }
                    }

                },
                Ok((protocol::Msg::Command(mut command), sender)) => {
                    println!("Got command message:\n\tid: {}\n\targs: {}", command.id, command.data.len());

                    if let Some((_, dispatch)) = modules.get(command.module_id as usize) {
                        let mut response_ptr: *mut u8 = std::ptr::null_mut();
                        let mut response_len = 0usize;
                        println!("calling dispatch");
                        unsafe { dispatch(command.id as usize, command.data.as_mut_ptr(), command.data.len(), &mut response_ptr, &mut response_len); }
                        let response = unsafe { Vec::from_raw_parts(response_ptr, response_len, response_len) };

                        let msg = protocol::Msg::Response(Response { data: response });
                        let mut response: Vec<u8> = Vec::new();
                        msg.encode(&mut response);
                        
                        let response_len = response.len();
                        match sender.send(response) {
                            Ok(_) => println!("Sent response of {} bytes", response_len),
                            Err(err) => println!("Failed to send response: {err}"),
                        }
                    }
                },
                Err(err) => {
                    println!("Recv failed: {err}");
                }
                _ => {
                    println!("Uexpected message type");
                }
            };
        }
    });

    threads.push(dispatch_thread);

    let mut server = Server::new();

    match server.bind("0.0.0.0:4444") {
        Ok(_) => println!("Server bound"),
        Err(err) => panic!("Failed to start listener: {err}"),
    }


    loop {
        let mut remote = match server.accept() {
            Ok(remote) => remote,
            Err(err) => {
                eprintln!("Failed to accept connection from remote: {err}");
                continue;
            }
        };

        let sender = dispatch_in_send.clone();
        
        let client_thread = std::thread::spawn(move || {
            //let receiver = dispatch_out_recv;

            let (response_send, response_recv) = std::sync::mpsc::channel::<Vec<u8>>();


            loop {
                let envelope = match remote.next_envelope() {
                    Ok(envelope) => envelope,
                    Err(err) => {
                        eprintln!("Failed to get next message: {err}");
                        break;
                    }
                };
                println!("Unpacked envelope");

                let message: protocol::Msg = match envelope.try_into() {
                    Ok(message) => message,
                    Err(err) =>{
                        eprintln!("Failed to decode message: {err}");
                        break;
                    }
                };
                println!("Decoded message");

                if let Err(err) = sender.send((message, response_send.clone())) {
                    eprintln!("Failed to send message to dispatch: {err}");
                    break
                }
                println!("Sent to dispatch");

                match response_recv.recv() {
                    Ok(response) => {
                        if let Err(err) = remote.wrap_and_send_envelope(response) {
                            eprintln!("Failed to send response to remote: {err}");
                        }
                        println!("Response sent");
                    },
                    Err(err) => {
                        eprintln!("Failed to recv response from dispatch: {err}");
                    }
                }
            }
        });

        threads.push(client_thread);
    }



}
