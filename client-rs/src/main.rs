
use wasmtime::component::{bindgen, Component, Linker, ResourceTable};
use wasmtime::*;
use wasmtime_wasi::p2::bindings::sync::Command;
use wasmtime_wasi::p2::{IoView, WasiCtx, WasiCtxBuilder, WasiView};

struct MyState {
    ctx: WasiCtx,
    table: ResourceTable,
}
impl IoView for MyState {
    fn table(&mut self) -> &mut ResourceTable { &mut self.table }
}
impl WasiView for MyState {
    fn ctx(&mut self) -> &mut WasiCtx { &mut self.ctx }
}

fn main() -> anyhow::Result<()> {
    let mut config = Config::new();  
    config.wasm_component_model(true);  
    config.debug_info(true);  
    let engine = Engine::new(&config)?;

    let bytes = std::fs::read(r#"target\wasm32-wasip2\debug\module_survey_wasm.wasm"#)?;
    let component = wasmtime::component::Component::new(&engine, bytes)?;

    let mut linker = wasmtime::component::Linker::<MyState>::new(&engine);
    wasmtime_wasi::p2::add_to_linker_sync(&mut linker)?;

    let mut builder = WasiCtxBuilder::new();

    let mut store = Store::new(
        &engine,
        MyState {
            ctx: builder
                .inherit_network()
                .inherit_stdio()
                .build(),
            table: ResourceTable::new(),
        },
    );

    bindgen!({
        world: "whisper-module",
        path: r#".\wit\module.wit"#,
    });

    let bindings = WhisperModule::instantiate(&mut store, &component, &linker)?;

    let module_descriptor = bindings.call_get_module_descriptor(&mut store)?;

    println!("module_descriptor: {module_descriptor:?}");

    bindings.call_handle_command(&mut store, 1, "command", &["arg1".to_owned(), "arg2".to_owned()])?;

    Ok(())
}
