use std::{any, path};

use log::{error, info, warn};

use whisper_lib::{envelope, protocol};

mod client;
use client::Client;

mod error;

mod module_manager;
use module_manager::ModuleManager;

use rustyline::DefaultEditor;
use clap::Parser;


fn read_line() -> anyhow::Result<String> {
    let mut rl = DefaultEditor::new()?;
    rl.readline(">> ").map_err(|e| e.into())
}

#[derive(Parser, Debug)]
struct ClientArgs {
    #[arg(help = "Connection string in ADDR:PORT syntax")]
    endpoint: String,

    #[arg(short, long, long_help, help = "Directory to look for *.wasm modules")]
    module_dir: Option<path::PathBuf>,

    #[arg(help = "Path(s) to individual *.wasm modules", trailing_var_arg = true)]
    modules: Vec<path::PathBuf>,
}

#[derive(Parser)]
#[command(name = "load", about = "Load a module on the remote", long_about = None)]
struct LoadArgs {
    #[arg(index=1)]
    module: String
}

fn print_help(module_manager: &mut module_manager::ModuleManager) {
    println!("===== Built-ins =====");
    println!("load [MODULE]\tLoad a module into core\n");
    
    for module_descriptor in module_manager.get_module_descriptors().unwrap_or_default() {
        println!("===== Module: {} =====", module_descriptor.name);
        for func in module_descriptor.funcs.unwrap_or_default() {
            println!("{}\t{}", func.name, func.description.unwrap_or_default());
            for arg in func.args.unwrap_or_default() {
                let arg_name = if arg.required {
                    arg.name
                } else {
                    format!("[{}]", arg.name)
                };

                println!("   {}    {}", arg_name.to_uppercase(), arg.help.unwrap_or_default());
            }
        }
        println!();
    }
}

fn resolve_command_to_module(command: &str, module_manager: &mut module_manager::ModuleManager) -> Vec<(String, String)> {
    module_manager.get_module_descriptors()
        .unwrap_or_default()
        .iter().filter_map(|desc| {
            if let Some(funcs) = &desc.funcs {
                if let Some(func) = funcs.iter().find(|func| func.name == command) {
                    Some((desc.name.clone(), func.name.clone()))
                } else {
                    None
                }
            } else {
                None
            }
        }).collect()
}

fn main() -> anyhow::Result<()> {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let mut module_ids: std::collections::BTreeMap<String, u64> = std::collections::BTreeMap::new();

    let args = ClientArgs::parse();

    info!("Parsed args:\n{args:#?}");

    let mut client = Client::new();

    client.connect(args.endpoint)?;

    let (tx, rx) = flume::unbounded::<module_manager::Msg>();

    let handler_tx = tx.clone();
    let mut module_manager = ModuleManager::new(
        move |msg| {
            //info!("Received msg: {msg:#?}");
            let _ = handler_tx.send(msg);
        }
    )?;

    if let Some(module_dir) = args.module_dir {
        module_manager.add_modules(module_dir.as_path())?;
    }

    for module_path in args.modules {
        module_manager.add_module(module_path.as_path())?;
    }

    info!("===== Listing modules:");
    for descriptor in module_manager.get_module_descriptors()? {
        info!("===== {} =====", descriptor.name);
        if let Some(funcs) = descriptor.funcs {
            for func in &funcs {
                let default_description = String::default();
                let description = func.description.as_ref().unwrap_or(&default_description);
                info!("\t{}\t{}", func.name, description);
            }
        }
    }

    // info!("===== Calling: survey::hostname");
    // match module_manager.call_module_command("survey", "hostname", Vec::new()) {
    //     Ok(_) => info!("Successfully called call_module_command"),
    //     Err(err) => error!("Failed to call call_module_command: {err}")
    // }
    
    loop {
        let line = read_line()?;

        if line.len() == 0 {
            error!("line was empty");
            continue
        }

        let Some(args) = shlex::split(line.as_str()) else {
            error!("Couldn't shlex::split: {line}");
            continue
        };

        if args.len() == 0 {
            error!("shlex::split returned 0 len args");
            continue
        }

        let command = args[0].as_str();
        match command {
            "load" => {
                let load_args = match LoadArgs::try_parse_from(args) {
                    Ok(load_args) => load_args,
                    Err(err) => {
                        error!("Failed to parse args: {err}");
                        continue
                    }
                };

                info!("Loading module: {}", load_args.module);

                if let Some(module_data) = module_manager.get_module_data(&load_args.module) {
                    info!("Got module_data: {}", module_data.len());
                    
                    info!("Sending load request");
                    match client.load_module(module_data) {
                        Ok(protocol::Response{
                            header: Some(_),
                            response: Some(protocol::response::Response::Load(protocol::LoadResponse{module_id}))
                        }) => {
                            info!("Module [{}] loaded", load_args.module);
                            module_ids.insert(load_args.module, module_id);
                        },
                        Ok(response) => {
                            panic!("Received unexpected response: {response:#?}");
                        }
                        Err(err) => warn!("Failed to load module: {err}"),
                    }
                } else {
                    warn!("No module data returned for: {}", load_args.module);
                }
            },
            "quit" | "exit" => break,
            "help" | "?" => print_help(&mut module_manager),
            _ => {
                let matched_commands = resolve_command_to_module(command, &mut module_manager);

                match matched_commands.len() {
                    0 => {
                        warn!("Did not find command: {command}");
                    },
                    1 => {
                        let (module, command) = &matched_commands[0];
                        if let Err(err) = module_manager.call_module_command(module, command, args[1..].to_vec()) {
                            warn!("Calling {module}:{command} failed: {err}");
                        }

                        match rx.recv() {
                            Ok(msg) => {
                                //println!("rx.recv msg: {msg:#?}");
                                let Some(module_id) = module_ids.get(&msg.module_id) else {
                                    warn!("Received message from unmapped module id: {}", msg.module_id);
                                    break
                                };
                                let response = client.send_command(
                                    *module_id,
                                    msg.command_id,
                                    msg.tx_id,
                                    msg.data)?;

                                if let Some(response) = response.response {
                                    match response {
                                        protocol::response::Response::Command(command_response) => {
                                            if let Err(code) = module_manager.send_module_message(&module, msg.tx_id, command_response.data) {
                                                error!("module_manager.send_module_message failed with: {code}");
                                            }
                                        },
                                        _ => unreachable!()
                                    }
                                }

                                //info!("Got response:\n{response:#?}");
                            },
                            Err(err) => {
                                warn!("Failed to receive in receiver_thread: {err}");
                                break
                            }
                        }
                    },
                    _ => {
                        warn!("Multiple commands matched {command}. Try scoping your command like <module>:<command>, i.e. survey:hostname");
                    }
                }
            }
        }
    }

    Ok(())
}
