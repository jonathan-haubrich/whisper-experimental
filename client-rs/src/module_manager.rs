
use log::info;
use wasmtime::component::{bindgen, ResourceTable};
use wasmtime::*;
use wasmtime_wasi::p2::{IoView, WasiCtx, WasiCtxBuilder, WasiView};

use anyhow::Result;


bindgen!({
    world: "whisper-module",
    path: r#".\wit\module.wit"#,
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

impl ModuleManager {
    pub fn new<F: Fn(Msg) + Send + 'static>(message_out_handler: F) -> Result<Self> {
        let mut config = Config::new();  
        config.wasm_component_model(true);  
        config.debug_info(true);

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

    pub fn add_module(&mut self, path: &std::path::Path) -> Result<()> {
        let bytes = std::fs::read(path)?;
        
        let component = wasmtime::component::Component::new(&self.engine, bytes)?;

        let bindings = WhisperModule::instantiate(&mut self.store, &component, &self.linker)?;

        let name = bindings.call_get_module_descriptor(&mut self.store)
            .map(|desc| desc.name.clone())?;

        info!("Added module with name: {name}");

        self.bindings.insert(name, bindings);

        Ok(())
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
        if let Some(binding) = self.bindings.get(module) {
            info!("found bindings, calling call_handle_command");
            let tx_id: u64 = rand::random();
            binding.call_handle_command(&mut self.store, tx_id, command, &args)?;
        }

        Ok(())
    }
}