mod module_descriptor;
//use module_descriptor::{ModuleDescriptor, ModuleFunction};

wit_bindgen::generate!({
    world: "whisper-module"
});

struct Survey;

impl Guest for Survey {
fn get_module_descriptor() -> ModuleDescriptor {
        ModuleDescriptor{
            name: "survey".to_owned(),
            description: None,

            funcs: Some(vec![
                ModuleFunc{
                    name: "get_hostname".to_owned(),
                    description: None,

                    args: Some(vec![])
                }
            ])
        }
    }
    
    fn handle_command(tx_id:u64, command: String, args: Vec::<String>) {
        println!("tx_id {tx_id} command: {command} args: {args:?}");
    }
}

export!(Survey);