use std::{io::Write, os::windows::fs::OpenOptionsExt, path::Path, thread::JoinHandle};

use client_rs::module_manager::{self, Msg};
use clap::Parser;
use flume::{self, Receiver, Sender};
use prost::Message;

use crate::{module::{CommandParams, MessageParams}, upload::{BeginResponse, ChunkResponse, EndResponse}};

#[derive(clap::Parser)]
struct ThreadedModuleArgs {
    #[arg()]
    module_path: String
}

// -- Structs for orchestrating messages between module and client
pub mod module {
    pub struct CommandParams {
        // export handle-command: func(tx-id: u64, command: string, args: list<string>);
    
        pub tx_id: u64,
        pub command: String,
        pub args: Vec<String>,
    }
    
    pub struct MessageParams {
        // export message-in: func(tx-id: u64, msg: list<u8>) -> result<_, u32>;
    
        pub tx_id: u64,
        pub msg: Vec<u8>,
    }
    
    pub enum Message {
        Command(CommandParams),
        Message(MessageParams),
    }

}
// -- End structs for orchestrating messages between module and client

// -- "Protocol" for communications between module and client

pub mod upload {
    #[derive(Clone, PartialEq, prost::Message)]
    pub struct Begin {
        #[prost(message, tag="1")]
        pub dest: Option<String>,
        #[prost(bytes="vec", optional, tag="2")]
        pub data: Option<Vec<u8>>,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct Chunk {
        #[prost(uint64, tag="1")]
        pub seq_no: u64,
        #[prost(uint64, tag="2")]
        pub total: u64,
        #[prost(bytes="vec", tag="3")]
        pub data: Vec<u8>,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct End {
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct BeginResponse {
        #[prost(bool, tag="1")]
        pub success: bool,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct ChunkResponse {
        #[prost(uint64, tag="1")]
        pub seq_no: u64,
        #[prost(bool, tag="2")]
        pub success: bool,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct EndResponse {
        #[prost(bool, tag="1")]
        pub success: bool,}
    
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum _Message {
        #[prost(message, tag="1")]
        Begin(Begin),
        #[prost(message, tag="2")]
        Chunk(Chunk),
        #[prost(message, tag="3")]
        End(End),

        #[prost(message, tag="4")]
        BeginResponse(BeginResponse),
        #[prost(message, tag="5")]
        ChunkResponse(ChunkResponse),
        #[prost(message, tag="6")]
        EndResponse(EndResponse)
    }

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Message {
        #[prost(oneof="_Message", tags="1, 2, 3, 4, 5, 6")]
        pub message: Option<_Message>,
    }

}
// -- End "protocol" for communications between module and client

fn handle_upload_message(message: upload::Message, tx_id: u64, transactions: &mut std::collections::BTreeMap<u64, std::path::PathBuf>) -> upload::Message {
    
    let upload_message = message.message.unwrap();
    //println!("upload_message: {upload_message:#?}");

    match upload_message {
        upload::_Message::Begin(begin) => {
            let dest = std::path::PathBuf::from(begin.dest.unwrap());

            let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&dest) else {
                    return upload::Message{
                        message: Some(upload::_Message::BeginResponse(BeginResponse{
                            success: false
                        }))
                    }
                };

            if let Some(data) = begin.data {
                if file.write_all(&data).is_err() {
                    return upload::Message{
                        message: Some(upload::_Message::BeginResponse(BeginResponse{
                            success: false
                        }))
                    }
                }
            }

            transactions.insert(tx_id, dest);

            upload::Message{
                message: Some(upload::_Message::BeginResponse(BeginResponse{
                    success: true
                }))
            }
        },
        upload::_Message::Chunk(chunk) => {
            let Some(dest) = transactions.get(&tx_id) else {
                return upload::Message{
                    message: Some(upload::_Message::BeginResponse(BeginResponse{
                        success: false
                    }))
                }
            };

            let Ok(mut file) = std::fs::OpenOptions::new()
                .append(true)
                .open(dest) else {
                return upload::Message{
                    message: Some(upload::_Message::BeginResponse(BeginResponse{
                        success: false
                    }))
                }
            };

            if let Err(..) = file.write_all(&chunk.data) {
                return upload::Message{
                    message: Some(upload::_Message::BeginResponse(BeginResponse{
                        success: false
                    }))
                }
            }

            upload::Message{
                message: Some(upload::_Message::ChunkResponse(ChunkResponse{
                    seq_no: chunk.seq_no,
                    success: true
                }))
            }
        },
        upload::_Message::End(_) => {
            let Some(dest) = transactions.get(&tx_id) else {
                return upload::Message{
                    message: Some(upload::_Message::EndResponse(EndResponse{
                        success: false
                    }))
                }
            };

            println!("Upload for {} finished!", dest.to_string_lossy());

            transactions.remove(&tx_id);

            upload::Message{
                message: Some(upload::_Message::EndResponse(EndResponse{
                    success: true
                }))
            }
        }
        _ => unimplemented!()
    }


}

fn spawn_client_thread(client_recv: Receiver<module::Message>, module_send: Sender<module::Message>) -> JoinHandle<()> {
    // client represents the remote side of an UploadFile module
    // client waits for BeginUpload message
    // then maintains state in order to complete the file upload
    
    let mut transactions: std::collections::BTreeMap<u64, std::path::PathBuf> = std::collections::BTreeMap::new();
    
    std::thread::spawn(move || {
        loop {
            println!("client thread waiting for recv");
            match client_recv.recv() {
                Ok(message @ module::Message::Command(_)) => {
                    println!("Got Message::Command");
                    module_send.send(message).unwrap();
                },
                Ok(module::Message::Message(params)) => {
                    println!("Got Message::Message");

                    let upload_message = upload::Message::decode(params.msg.as_slice()).unwrap();

                    let response = handle_upload_message(upload_message, params.tx_id, &mut transactions);

                    let mut data = Vec::new();

                    response.encode(&mut data).unwrap();

                    module_send.send(
                        module::Message::Message(MessageParams{
                            tx_id: params.tx_id,
                            msg: data
                        })
                    ).unwrap();
                }
                Err(err) => {
                    eprintln!("client_thread failed to client_recv.recv: {err}");
                    return
                }
            }
        }
    })
}

fn spawn_module_thread(module_path: &String, module_recv: Receiver<module::Message>, client_send: Sender<module::Message>) -> JoinHandle<()> {
    let module_path = module_path.clone();
    std::thread::spawn(move || {
            let Ok(mut  manager) = module_manager::ModuleManager::new(move |msg| {
                println!("in message_out_handler");
                let message = module::Message::Message(
                    MessageParams{
                        tx_id: msg.tx_id,
                        msg: msg.data.unwrap()
                    }
                );
                _ = client_send.send(message).inspect_err(|e| eprintln!("Failed to send msg to client: {e}"));
                println!("client_send sent message");
            }) else {
                eprintln!("Failed to create ModuleManager");
                return;
            };

            let Ok(name) = manager.add_module(Path::new(&module_path)) else {
                return;
            };

            loop {
                match module_recv.recv() {
                    Ok(module_message) => {
                        match module_message {
                            module::Message::Command(params) => {
                                match manager.call_module_command(&name, &params.command, params.args) {
                                    Ok(_) => println!("call_module_command succeeded"),
                                    Err(err) => eprintln!("call_module_command failed: {err}"),
                                }
                            },
                            module::Message::Message(params) => {
                                match manager.send_module_message(&name, params.tx_id, params.msg) {
                                    Ok(_) => println!("send_module_message succeeded"),
                                    Err(code) => eprintln!("send_module_message failed: {code}"),
                                }
                            }
                        }
                    },
                    Err(err) => {
                        eprintln!("module thread failed to recv message: {err}");
                        break
                    } 
                }
            }
        })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = ThreadedModuleArgs::parse();

    let (module_send, module_recv) = flume::unbounded::<module::Message>();
    let (client_send, client_recv) = flume::unbounded::<module::Message>();

    module_send.send(module::Message::Command(CommandParams{
        tx_id: 1,
        command: "upload".into(),
        args: vec![r#"/Users/dweller/source/repos/poor-mans-rpc/bc5475b0-7c0a-4f2e-b7d8-0df85fecf091.bin"#.into(), "test_file_out".into()]
    })).unwrap();

    let module_thread = spawn_module_thread(&args.module_path, module_recv, client_send);
    let client_thread = spawn_client_thread(client_recv, module_send);

    _ = module_thread.join().inspect_err(|_| eprintln!("Failed to join module thread"));
    println!("module_thread joined");
    _ = client_thread.join().inspect_err(|_| eprintln!("Failed to join client thread"));
    println!("client_thread joined");

    Ok(())
}