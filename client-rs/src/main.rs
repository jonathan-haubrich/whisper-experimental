


mod module_manager;
use log::{error, info};
use module_manager::ModuleManager;

fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let path = r#"target\wasm32-wasip2\debug\module_survey_wasm.wasm"#;

    let mut module_manager = ModuleManager::new(
        |msg| info!("Received msg: {msg:#?}")
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

    Ok(())
}
