use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::OnceLock;

use crate::upload::state::UploadState;

mod state;

use crate::protocol::file::upload::{Upload, upload};

type UploadStates = HashMap<u64, state::UploadState>;
struct UploadTransactions(RefCell<UploadStates>);

// SAFETY: wasm components are only ever run in a single thread
unsafe impl Sync for UploadTransactions {}

static UPLOADS: OnceLock<UploadTransactions> = OnceLock::new();

const CHUNK_SIZE: usize = 0x2000;

pub(crate) fn handle_upload_command(tx_id: u64, args: Vec<String>) -> anyhow::Result<upload::Request> {
    // validate args
    // upload requires a source and destination
    let [source, dest] = &args[0..2] else {
        return Err(anyhow::anyhow!("Invalid upload arguments"))
    };

    let mut uploads = uploads().borrow_mut();

    let upload = uploads.get(&tx_id);
    match upload {
        Some(state::UploadState::Uploading(..)) => {
            return Err(anyhow::anyhow!("Upload in progress, cannot start new upload for tx_id: {tx_id}"))
        },
        _ => {}
    }

    let mut uploading = state::UploadingState::new(&source)?;

    let chunk = uploading.next_chunk(CHUNK_SIZE).unwrap_or_default();

    uploads.insert(tx_id, state::UploadState::Uploading(uploading));

    Ok(upload::Request{
        message: Some(
            upload::request::Message::Begin(
                upload::request::Begin{
                    tx_id,
                    dest: dest.clone(),
                    chunk,
                }
            )
        )
    })
}

fn handle_response_message(tx_id: u64, response: upload::response::Message) -> anyhow::Result<Option<upload::Message>> {
    let mut uploads = uploads().borrow_mut();

    let Some(UploadState::Uploading(upload)) = uploads.get_mut(&tx_id) else {
        return Err(anyhow::anyhow!("No upload found for transaction: {tx_id}"));
    };
    
    match response {
        upload::response::Message::Begin(begin) => {
            if begin.error != 0 {
                return Err(anyhow::anyhow!("Could not begin upload: {}", begin.error));
            }

            let chunk = upload.next_chunk(CHUNK_SIZE).unwrap_or_default();

            Ok(
                Some(
                    upload::Message::Request(
                        upload::Request{
                            message: Some(
                                upload::request::Message::Chunk(
                                    upload::request::Chunk{
                                        tx_id,
                                        seq_no: 0,
                                        total: 0,
                                        chunk
                                    }
                                ) 
                            )
                        }
                    )
                )
            )
        },
        upload::response::Message::Chunk(chunk) => {
            if chunk.error != 0 {
                return Err(anyhow::anyhow!("Error received during upload: {}", chunk.error));
            }

            if upload.uploaded >= upload.len {
                return Ok(
                    Some(
                        upload::Message::Request(
                            upload::Request{
                                message: Some(upload::request::Message::End(
                                    upload::request::End{
                                        tx_id
                                    }
                                ))
                            }
                        )
                    )
                )
            }

            let chunk = upload.next_chunk(CHUNK_SIZE).unwrap_or_default();

            Ok(
                Some(
                    upload::Message::Request(
                        upload::Request{
                            message: Some(upload::request::Message::Chunk(
                                    upload::request::Chunk{
                                        tx_id,
                                        seq_no: 0,
                                        total: 0,
                                        chunk
                                    }
                                ) 
                            )
                        }
                    )
                )
            )
        },
        upload::response::Message::End(end) => {
            if end.error != 0 {
                return Err(anyhow::anyhow!("Error received during upload: {}", end.error));
            }

            Ok(None)
        }
    }
}

pub(crate) fn handle_upload_message(tx_id: u64, upload: Upload) -> anyhow::Result<Option<upload::Message>> {
    // we expect to receive response messages and send request messages
    match upload.message {
        Some(
            upload::Message::Response(upload::Response{
                message: Some(
                    response
                )
            })
        ) => {
            return handle_response_message(tx_id, response);
        },
        _ => todo!()
    }
}

fn uploads() -> &'static RefCell<UploadStates> {
    &UPLOADS.get_or_init(|| UploadTransactions(RefCell::new(UploadStates::new()))).0
}