use std::any;

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

#[derive(Parser)]
#[command(name = "load", about = "Load a module on the remote", long_about = None)]
struct LoadArgs {
    #[arg(short, long)]
    module: String
}

fn main() -> anyhow::Result<()> {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Info)
        .init();    

    let mut client = Client::new();

    client.connect("127.0.0.1:4444")?;

    let (tx, rx) = flume::unbounded();

    let path = r#"target\wasm32-wasip2\debug\module_survey_wasm.wasm"#;

    let handler_tx = tx.clone();
    let mut module_manager = ModuleManager::new(
        move |msg| {
            info!("Received msg: {msg:#?}");
            let _ = handler_tx.send(msg.data);
        }
    )?;

    info!("===== Listing modules:");
    for descriptor in module_manager.get_module_descriptors()? {
        info!("Descriptor for: {}\n{:#?}", descriptor.name, descriptor);
    }

    info!("Adding module: {path}");
    module_manager.add_module(&std::path::Path::new(path))?;

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

    info!("===== Calling: survey::hostname");
    match module_manager.call_module_command("survey", "hostname", Vec::new()) {
        Ok(_) => info!("Successfully called call_module_command"),
        Err(err) => error!("Failed to call call_module_command: {err}")
    }
    
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

        match args[0].as_str() {
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
                } else {
                    warn!("No module data returned for: {}", load_args.module);
                }

            },
            "quit" | "exit" => break,
            _ => warn!("Unrecognized command: {}", args[0])
        }
    }

    Ok(())
}
