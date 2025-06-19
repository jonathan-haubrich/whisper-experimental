use std::collections::HashMap;
use std::cell::RefCell;
use std::io::Read;
use std::sync::OnceLock;
use std::u32;

use prost::Message;


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
pub struct FileMessage {
    #[prost(oneof="_Message", tags="1, 2, 3, 4, 5, 6")]
    pub message: Option<_Message>,
}

#[derive(Debug)]
struct Uploading {
    pub file: std::fs::File,
    pub uploaded: u64,
    pub len: u64,

    current_chunk: u64,
}

impl Uploading {
    pub fn new(path: &str) -> Result<Self, std::io::Error> {
        println!("Opening path: {path:#?}");
        let file = std::fs::File::open(path)?;

        let file_len = file.metadata().unwrap().len();

        Ok(Self {
            file,
            uploaded: 0,
            len: file_len,
            current_chunk: 0
        })
    }
    pub fn next_chunk(&mut self) -> Option<(u64, Vec<u8>)> {
        if self.uploaded >= self.len {
            return None;
        }

        let chunk_len = std::cmp::min(CHUNK_SIZE, self.len - self.uploaded) as usize;
        let mut chunk = vec![0u8; chunk_len];

        match self.file.read(&mut chunk) {
            Ok(bytes_read) => {
                self.uploaded += bytes_read as u64;
                let this_chunk = self.current_chunk;
                self.current_chunk += 1;

                Some((this_chunk, chunk))
            },
            Err(_) => None
        }
    }
}

#[derive(Debug)]
enum UploadState {
    Unstarted,
    Uploading(Uploading),
    Finished,
    Errored,
}

wit_bindgen::generate!({
    world: "whisper-module",
    path: r#"..\wit\module.wit"#,
});

static MODULE_NAME: &str = "survey";

const CHUNK_SIZE: u64 = 0x2000;

struct File;

struct UploadTransactions(RefCell<HashMap<u64, UploadState>>);

static UPLOADS: OnceLock<UploadTransactions> = OnceLock::new();

// SAFETY: wasm components are only ever run in a single thread
unsafe impl Sync for UploadTransactions {}


impl Guest for File {
    fn message_in(tx_id: u64, msg: Vec::<u8>) -> Result<(), u32> {
            let message = FileMessage::decode(msg.as_slice()).map_err(|_| u32::MAX)?;

            match message.message {
                Some(_Message::BeginResponse(response)) => {
                    if response.success {
                        let message = File::handle_upload(tx_id, None, None).map_err(|_| u32::MAX)?;

                        let mut data = Vec::new();

                        message.encode(&mut data);

                        let msg = &Msg { module_id: MODULE_NAME.to_owned(), command_id: 0, tx_id: tx_id, data: Some(data)};

                        message_out(msg);
                    }
                },
                Some(_Message::ChunkResponse(response)) => {
                    if response.success {
                        let message = File::handle_upload(tx_id, None, None).map_err(|_| u32::MAX)?;

                        let mut data = Vec::new();

                        message.encode(&mut data);

                        let msg = &Msg { module_id: MODULE_NAME.to_owned(), command_id: 0, tx_id: tx_id, data: Some(data)};

                        message_out(msg);
                    }
                },
                Some(_Message::EndResponse(response)) => {
                    if response.success {
                        println!("File uploaded successfully");
                    }
                }
                _ => unreachable!(),
            }

            Ok(())
        }

    fn get_module_descriptor() -> ModuleDescriptor {
            ModuleDescriptor{
                name: MODULE_NAME.into(),
                description: None,

                funcs: Some(vec![
                    ModuleFunc{
                        name: "upload".to_owned(),
                        description: Some("Upload a file to remote".to_owned()),

                        args: Some(vec![
                            ModuleFuncArg{
                                name: "source".into(),
                                type_: "String".into(),
                                required: true,
                                help: Some("Source path for local file to upload".into()),
                            },
                            ModuleFuncArg{
                                name: "source".into(),
                                type_: "String".into(),
                                required: true,
                                help: Some("Destination path to which remote file will be saved".into()),
                            },
                        ])
                    }
                ])
            }
        }

    fn get_module_data() -> Vec::<u8> {
            Vec::new()
        }

    fn handle_command(tx_id: u64, command: String, args: Vec::<String>) {
            println!("handling command: tx_id: {tx_id} command: {command} args: {args:#?}");
            match command.as_str() {
                "upload" => {
                    let message = File::handle_upload(tx_id, Some(args[0].clone()), Some(args[1].clone())).unwrap();

                    let mut data = Vec::new();

                    message.encode(&mut data);

                    let msg = &Msg { module_id: MODULE_NAME.to_owned(), command_id: 0, tx_id: tx_id, data: Some(data)};

                    message_out(msg);
                },
                _ => println!("Unrecognized command: {command}")
            }
        }
}

impl File {
    // handles upload commands
    // implemented as a state machine
    fn handle_upload(tx_id: u64, source: Option<String>, dest: Option<String>) -> Result<_Message, std::io::Error> {
        println!("in module_file::handle_upload");
        let mut transactions = File::get_uploads().borrow_mut();

        let entry = transactions.entry(tx_id).or_insert(UploadState::Unstarted);

        let message = match entry {
            UploadState::Unstarted => {
                let Some(source) = source else {
                    transactions.insert(tx_id, UploadState::Errored);
                    eprintln!("source path not provided to upload command");
                    return Err(std::io::Error::new(std::io::ErrorKind::Other, "source path not provided to upload command"));
                };

                println!("Calling Uploading::new");
                let mut uploading = Uploading::new(&source)?;
                println!("Called Uploading::new");

                let chunk = uploading.next_chunk().map(|c| c.1);

                transactions.insert(tx_id, UploadState::Uploading(uploading));

                _Message::Begin(Begin { dest, data: chunk })
            },
            UploadState::Uploading(uploading) => {
                if uploading.uploaded >= uploading.len {
                    transactions.insert(tx_id, UploadState::Finished);

                    _Message::End(End { })
                } else {
                    let (seq_no, chunk) = match uploading.next_chunk() {
                        Some(next_chunk) => next_chunk,
                        None => (u64::MAX, Vec::new())
                    };
    
                    let total = uploading.len;
    
                    _Message::Chunk(Chunk { seq_no, total, data: chunk })
                }
            },
            _ => return Err(std::io::Error::new(std::io::ErrorKind::Other, "something went wrong"))
        };

        Ok(message)
    }

    fn get_uploads() -> &'static RefCell<HashMap<u64, UploadState>> {
        &UPLOADS.get_or_init(|| UploadTransactions(RefCell::new(HashMap::new()))).0
    }
}

export!(File);