
use std::path::PathBuf;
use std::u32;

use log::{info, warn};
use wasmtime::component::{bindgen, ResourceTable};
use wasmtime::*;
use wasmtime_wasi::p2::{IoView, WasiCtx, WasiCtxBuilder, WasiView};

use anyhow::Result;
use wasmtime_wasi::{DirPerms, FilePerms};

bindgen!({
    world: "whisper-module",
    path: r#"..\wit\module.wit"#,
});

pub struct ModuleManager {
    pub engine: wasmtime::Engine,
    pub linker: wasmtime::component::Linker<MyState>,
    pub store: wasmtime::Store<MyState>,
    pub bindings: std::collections::BTreeMap<String, WhisperModule>,
}

pub struct MyState {
    ctx: WasiCtx,
    table: ResourceTable,
    message_out_handler: Box<dyn Fn(Msg) + Send>,
}
impl IoView for MyState {
    fn table(&mut self) -> &mut ResourceTable { &mut self.table }
}
impl WasiView for MyState {
    fn ctx(&mut self) -> &mut WasiCtx { &mut self.ctx }
}

impl WhisperModuleImports for MyState {
    fn message_out(&mut self, msg: Msg) {
        (self.message_out_handler)(msg);
    }
}

fn get_filesystem_root() -> Result<std::path::PathBuf, std::io::Error> {
    let current_dir = std::env::current_dir()?;

    println!("current dir: {current_dir:#?}");

    let root = current_dir.ancestors().last().unwrap();

    println!("root: {root:#?}");
    
    Ok(PathBuf::from(root))
}

impl ModuleManager {
    pub fn new<F: Fn(Msg) + Send + 'static>(message_out_handler: F) -> Result<Self> {
        let mut config = Config::new();  
        config.wasm_component_model(true);  
        config.debug_info(true);
        config.wasm_threads(true);
        config.wasm_shared_everything_threads(true);

        let engine = Engine::new(&config)?;
        let mut linker = wasmtime::component::Linker::<MyState>::new(&engine);
        WhisperModule::add_to_linker(&mut linker, |state: &mut MyState| state)?;

        wasmtime_wasi::p2::add_to_linker_sync(&mut linker)?;

        let mut builder = WasiCtxBuilder::new();

        let store = Store::new(
            &engine,
            MyState {
                ctx: builder
                    .inherit_network()
                    .inherit_stdio()
                    .preopened_dir(r#"C:\"#, r#"/"#, DirPerms::all(), FilePerms::all())?
                    .build(),
                table: ResourceTable::new(),
                message_out_handler: Box::new(message_out_handler),
            },
        );

        Ok(Self {
            engine,
            linker,
            store,
            bindings: std::collections::BTreeMap::new(),
        })
    }

    pub fn add_modules(&mut self, path: &std::path::Path) -> Result<()> {
        for dir_entry_result in std::fs::read_dir(path)? {
            if let Ok(dir_entry) = dir_entry_result {
                let module_path = dir_entry.path();
                let has_wasm_extension = module_path.extension().map(|ext| ext == "wasm").unwrap_or(false);
                if !has_wasm_extension {
                    continue;
                }

                info!("Attempting to load module: {module_path:#?}");

                if let Err(err) = self.add_module(&module_path) {
                    warn!("Failed to load module [{}]: {err}", module_path.to_string_lossy());
                }
            }
        }

        Ok(())
    }

    pub fn add_module(&mut self, path: &std::path::Path) -> Result<String> {
        let bytes = std::fs::read(path)?;
        
        let component = wasmtime::component::Component::new(&self.engine, bytes)?;

        let bindings = WhisperModule::instantiate(&mut self.store, &component, &self.linker)?;

        let name = bindings.call_get_module_descriptor(&mut self.store)
            .map(|desc| desc.name.clone())?;

        info!("Added module with name: {name}");

        self.bindings.insert(name.clone(), bindings);

        Ok(name)
    }

    pub fn get_module_descriptors(&mut self) -> Result<Vec<ModuleDescriptor>> {
        let mut module_descriptors = Vec::new();

        for binding in self.bindings.values() {
            let module_descriptor = binding.call_get_module_descriptor(&mut self.store)?;

            module_descriptors.push(module_descriptor);
        }

        Ok(module_descriptors)
    }

    pub fn get_module_descriptor(&mut self, module: &str) -> Option<ModuleDescriptor> {
        if let Some(binding) = self.bindings.get(module) {
            return binding.call_get_module_descriptor(&mut self.store)
                .map(|desc| Some(desc))
                .unwrap_or(None);
        }

        None
    }

    pub fn get_module_data(&mut self, module: &str) -> Option<Vec<u8>> {
        if let Some(binding) = self.bindings.get(module) {
            return binding.call_get_module_data(&mut self.store)
                .map(|data| Some(data))
                .unwrap_or(None);
        }

        None
    }

    pub fn call_module_command(&mut self, module: &str, command: &str, args: Vec<String>) -> Result<()> {
        info!("module: {module} command: {command}");
        match self.bindings.get(module) {
            Some(binding) => {
                info!("found bindings, calling call_handle_command");
                let tx_id: u64 = rand::random();
                binding.call_handle_command(&mut self.store, tx_id, command, &args)?;
            },
            None => return Err(anyhow::anyhow!("Module not found: {module}")),
        }

        Ok(())
    }

    pub fn send_module_message(&mut self, module: &str, tx_id: u64, data: Vec<u8>) -> Result<(), u32> {
        info!("module: {module} tx_id: {tx_id} len(data): {}", data.len());

        match self.bindings.get(module) {
            Some(binding) => {
                info!("found bindings, calling call_message_in");
                _ = binding.call_message_in(&mut self.store, tx_id, &data).or(Err(u32::MAX))?;
            },
            None => return Err(u32::MAX)
        }

        Ok(())
    }
}