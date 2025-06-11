
wit_bindgen::generate!({
    world: "whisper-module",
    path: r#"..\wit\module.wit"#,
});

static MODULE_NAME: &str = "survey";

struct Survey;

impl Guest for Survey {
    fn get_module_descriptor() -> ModuleDescriptor {
        println!("inside get_module_descriptor");
        ModuleDescriptor{
            name: MODULE_NAME.into(),
            description: None,

            funcs: Some(vec![
                ModuleFunc{
                    name: "hostname".to_owned(),
                    description: Some("Gets the hostname of remote".to_owned()),

                    args: None
                }
            ])
        }
    }
    
    fn handle_command(tx_id:u64, command: String, args: Vec::<String>) {
        println!("tx_id {tx_id} command: {command} args: {args:?}");

        let handle = std::thread::spawn(|| {
            println!("Inside thread...");
        });

        let _ = handle.join();

        match command.as_str() {
            "hostname" => Survey::hostname(tx_id),
            _ => eprintln!("Unrecognized command: {command}"),
        }
    }

    fn message_in(tx_id: u64, message: Vec<u8>) -> Result<(), u32> {
        println!("tx_id {tx_id} message: {message:#?}");

        // message_in is going to be a response from a `handle_command`
        //
        // we will need to track tx_id -> some function state

        Ok(())
    }
    
    #[allow(async_fn_in_trait)]
    fn get_module_data() -> Vec::<u8> {
        let bytes = include_bytes!(env!("MODULE_DLL_FILEPATH"));

        bytes.to_vec()
    }
}

impl Survey {
    fn hostname(tx_id: u64) {
        const HOSTNAME_COMMAND_ID: u64 = 0;
        println!("hostname called");

        message_out(&Msg {
            module_id: MODULE_NAME.into(),
            command_id: HOSTNAME_COMMAND_ID,
            tx_id,
            data: None,
        });
    }
}

export!(Survey);