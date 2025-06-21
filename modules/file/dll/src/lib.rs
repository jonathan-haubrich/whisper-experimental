use std::{io::Write, u64};

use prost::Message;

use crate::protocol::file::upload::upload::{request, response};

pub mod protocol {
    pub mod file {
        include!(r#"codegen\whisper.module.file.rs"#);
        pub mod upload {
            include!(r#"codegen\whisper.module.file.upload.rs"#);
        }
    }
}


struct UploadStates(std::cell::RefCell<std::collections::HashMap<u64, std::fs::File>>);

unsafe impl Sync for UploadStates {}

// SAFETY: lol none, just testing. gl 🫡
static UPLOADS: std::sync::OnceLock<UploadStates> = std::sync::OnceLock::new();

//
// pub type FnDispatch = unsafe extern "C" fn(id: usize, arg_ptr: *mut u8, arg_len: usize, ret_ptr: &mut *mut u8, ret_len: &mut usize);
//
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dispatch(id: usize, arg_ptr: *mut u8, arg_len: usize, ret_ptr: &mut *mut u8, ret_len: &mut usize) {
    let args = unsafe { Vec::from_raw_parts(arg_ptr, arg_len, arg_len) };
    let args = std::boxed::Box::leak(args.into_boxed_slice());

    let message = match id {
        0 => handle_upload_message(args),
        _ => unimplemented!()
    };

    let mut response = Vec::new();
    _ = message.encode(&mut response);

    *ret_ptr = response.as_mut_ptr();
    *ret_len = response.len();

    std::mem::forget(response);
}

fn upload_response_new(message: protocol::file::upload::upload::response::Message) -> protocol::file::File {
    protocol::file::File{
        message: Some(protocol::file::file::Message::Upload(
            protocol::file::upload::Upload{
                message: Some(protocol::file::upload::upload::Message::Response(
                    protocol::file::upload::upload::Response{
                        message: Some(message)
                    }
                ))
            }
        ))
    }
}

fn handle_upload_message(args: &[u8]) -> protocol::file::File {
    let message = match protocol::file::File::decode(args) {
        Ok(message) => message,
        Err(err) => {
            eprintln!("Failed to decode file message: {err}");
            return upload_response_new(protocol::file::upload::upload::response::Message::Begin(
                protocol::file::upload::upload::response::Begin{
                    tx_id: 0,
                    error: u64::MAX,
                }
            ));
        }
    };

    let mut uploads = uploads().borrow_mut();

    match message.message {
        Some(protocol::file::file::Message::Upload(
            protocol::file::upload::Upload{
                message: Some(protocol::file::upload::upload::Message::Request(
                    protocol::file::upload::upload::Request{
                        message: Some(request)
                    }
                )) 
            }
        )) => {
            match request {
                request::Message::Begin(begin) => {
                    println!("Got Message::Begin");
                    if let Some(_) = uploads.get(&begin.tx_id) {
                        eprintln!("Upload for transaction [{}] already in progress...", begin.tx_id);
                        return upload_response_new(protocol::file::upload::upload::response::Message::Begin(
                            protocol::file::upload::upload::response::Begin{
                                tx_id: begin.tx_id,
                                error: u64::MAX,
                            }
                        ));
                    }

                    let mut file = match std::fs::OpenOptions::new()
                        .create(true)
                        .write(true)
                        .truncate(true)
                        .open(&begin.dest) {
                            Ok(file) => file,
                            Err(err) => {
                                eprintln!("Failed to open file [{}]: {err}", begin.dest);
                                return upload_response_new(protocol::file::upload::upload::response::Message::Begin(
                                    protocol::file::upload::upload::response::Begin{
                                        tx_id: begin.tx_id,
                                        error: u64::MAX,
                                    }
                                ));                            }
                        };

                    if let Err(err) = file.write_all(&begin.chunk) {
                        eprintln!("Failed to write to file [{}]: {err}", begin.dest);
                        return upload_response_new(protocol::file::upload::upload::response::Message::Begin(
                            protocol::file::upload::upload::response::Begin{
                                tx_id: begin.tx_id,
                                error: u64::MAX,
                            }
                        ));  
                    }

                    upload_response_new(protocol::file::upload::upload::response::Message::Begin(
                        protocol::file::upload::upload::response::Begin{
                            tx_id: begin.tx_id,
                            error: 0,
                        }
                    ))
                },
                request::Message::Chunk(chunk) => {
                    println!("Got Message::Chunk");
                    let Some(upload) = uploads.get_mut(&chunk.tx_id) else {
                        eprintln!("No upload found for transaction [{}]", chunk.tx_id);
                        return upload_response_new(protocol::file::upload::upload::response::Message::Chunk(
                            protocol::file::upload::upload::response::Chunk{
                                tx_id: chunk.tx_id,
                                seq_no: 0,
                                error: u64::MAX,
                            }
                        ));
                    };

                    if let Err(err) = upload.write_all(chunk.chunk.as_slice()) {
                        eprintln!("Failed to write chunk for transaction [{}]: {err}", chunk.tx_id);
                        return upload_response_new(protocol::file::upload::upload::response::Message::Chunk(
                            protocol::file::upload::upload::response::Chunk{
                                tx_id: chunk.tx_id,
                                seq_no: 0,
                                error: u64::MAX,
                            }
                        ));
                    }

                    upload_response_new(protocol::file::upload::upload::response::Message::Chunk(
                        protocol::file::upload::upload::response::Chunk{
                            tx_id: chunk.tx_id,
                            seq_no: 0,
                            error: 0,
                        }
                    ))
                },
                request::Message::End(end) => {
                    let Some(_) = uploads.remove(&end.tx_id) else {
                        eprintln!("No upload found for transaction [{}]", end.tx_id);
                        return upload_response_new(protocol::file::upload::upload::response::Message::End(
                            protocol::file::upload::upload::response::End{
                                tx_id: end.tx_id,
                                error: u64::MAX,
                            }
                        ));
                    };

                    upload_response_new(protocol::file::upload::upload::response::Message::End(
                        protocol::file::upload::upload::response::End{
                            tx_id: end.tx_id,
                            error: 0,
                        }
                    ))
                }
            }
        },
        _ => unimplemented!()
        
    }
}

fn uploads() -> &'static std::cell::RefCell<std::collections::HashMap::<u64, std::fs::File>> {
    &UPLOADS.get_or_init(|| UploadStates(std::cell::RefCell::new(std::collections::HashMap::<u64, std::fs::File>::new()))).0
}

#[cfg(test)]
mod tests {
}
