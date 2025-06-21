use memory_module::{allocator::VirtualAlloc, FnDispatch, MemoryModule};
use prost::Message;
use std::sync::mpsc;
use std::thread;

use server::Server;

mod server;
mod remote;

use whisper_lib::{envelope, protocol};

fn main() {
    pretty_env_logger::formatted_builder()
    .filter_level(log::LevelFilter::Info)
    .init(); 

    let (dispatch_in_send, dispatch_in_recv)= mpsc::channel::<(protocol::Request, mpsc::Sender::<Vec<u8>>)>(); 

    let mut threads: Vec<thread::JoinHandle<()>> = Vec::new();

    let dispatch_thread = std::thread::spawn(move || {
        let receiver = dispatch_in_recv;

        let mut modules: Vec<(MemoryModule<VirtualAlloc>, FnDispatch)> = Vec::new();

        loop {

            match receiver.recv() {
                Ok((protocol::Request { header: Some(header), request: Some(protocol::request::Request::Load(load)), }, sender)) => {
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
                                let module_id: u64 = modules.len() as u64 - 1;

                                let response = protocol::Response{
                                    header: Some(protocol::Header{
                                        tx_id: header.tx_id,
                                    }),
                                    response: Some(protocol::response::Response::Load(
                                        protocol::LoadResponse {
                                            module_id: module_id 
                                        })
                                    )
                                }.encode_to_vec();

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
                Ok((protocol::Request { header: Some(header), request: Some(protocol::request::Request::Command(mut command)), }, sender)) => {
                    println!("Got command message:\n\tid: {}\n\targs: {}", command.id, command.data.len());

                    if let Some((_, dispatch)) = modules.get(command.module_id as usize) {
                        let mut command_result_ptr: *mut u8 = std::ptr::null_mut();
                        let mut command_result_len = 0usize;
                        
                        println!("calling dispatch");
                        unsafe {
                            dispatch(
                                command.id as usize,
                                command.data.as_mut_ptr(),
                                command.data.len(),
                                &mut command_result_ptr,
                                &mut command_result_len
                            );
                        }

                        // TODO: We should forget this memory? Or give it back over ffi for freeing
                        let command_result = unsafe { 
                            Vec::from_raw_parts(
                                command_result_ptr,
                                command_result_len,
                                command_result_len
                            )
                        };

                        let response = protocol::Response {
                            header: Some(protocol::Header{
                                tx_id: header.tx_id,
                            }),
                            response: Some(protocol::response::Response::Command(
                                protocol::CommandResponse {
                                    data: command_result
                                })
                            )
                        }.encode_to_vec();

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

                let request = match envelope.try_into() {
                    Ok(request) => request,
                    Err(err) =>{
                        eprintln!("Failed to decode message: {err}");
                        break;
                    }
                };
                println!("Decoded message");

                if let Err(err) = sender.send((request, response_send.clone())) {
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
