use core::sync;
use std::u32;

use prost::Message;

use crate::upload::handle_upload_message;

pub mod upload;

pub mod protocol{
    pub mod file {
        include!(r#"codegen\whisper.module.file.rs"#);
        
        pub mod upload {
            include!(r#"codegen\whisper.module.file.upload.rs"#);
        }
    }
}

wit_bindgen::generate!({
    world: "whisper-module",
    path: r#"..\..\..\wit\module.wit"#,
});

static MODULE_NAME: &str = "file";

struct File;

impl Guest for File {
    fn message_in(tx_id: u64, msg: Vec::<u8>) -> Result<(), u32> {
        println!("inside message_in");
        let Ok(file_message) = protocol::file::File::decode(msg.as_slice()) else {
            eprintln!("Failed to decode message");
            return Err(u32::MAX);
        };

        if let Some(message) = file_message.message {
            println!("matching message");
            match message {
                protocol::file::file::Message::Upload(upload) => {
                    if let Ok(Some(request)) = handle_upload_message(tx_id, upload) {
                        println!("handle_upload_message succeeded");

                        let message = protocol::file::File{
                            message: Some(protocol::file::file::Message::Upload(
                                protocol::file::upload::Upload{
                                    message: Some(request)
                                }
                            ))
                        };

                        let mut data = Vec::new(); 
                        if message.encode(&mut data).is_err() {
                            eprintln!("[upload] Failed to encode response, upload failed")
                        }

                        println!("calling message_out");
                        message_out(&Msg{
                            tx_id,
                            module_id: MODULE_NAME.into(),
                            command_id: 0,
                            data: Some(data)
                        })
                    }
                },
                _ => {
                    eprintln!("Received unexpected message type: {message:#?}");
                }
            }
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
                                name: "dest".into(),
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
        let bytes = include_bytes!(env!("MODULE_DLL_FILEPATH"));

        println!("Returning module data with len: {}", bytes.len());

        bytes.to_vec()
    }

    fn handle_command(tx_id: u64, command: String, args: Vec::<String>) {
        match command.as_str() {
            "upload" => {
                match upload::handle_upload_command(tx_id, args) {
                    Ok(request) => {
                        let message = protocol::file::File{
                            message: Some(protocol::file::file::Message::Upload(
                                protocol::file::upload::Upload{
                                    message: Some(protocol::file::upload::upload::Message::Request(request))
                                }
                            ))
                        };

                        let mut data = Vec::new(); 
                        if message.encode(&mut data).is_err() {
                            eprintln!("[upload] Failed to encode response, upload failed")
                        }

                        message_out(&Msg{
                            tx_id,
                            module_id: MODULE_NAME.into(),
                            command_id: 0,
                            data: Some(data)
                        })
                    },
                    Err(err) => eprintln!("[upload] Upload failed: {err}"),
                }
            },
            _ => unimplemented!()
        }
    }
}
export!(File);

#[cfg(test)]
mod tests {
}
