use std::io::Read;

use indexmap::IndexMap;
use prost::Message;
use prost_types::{compiler::{CodeGeneratorRequest, CodeGeneratorResponse}, source_code_info::Location, SourceCodeInfo};

#[derive(Default, serde::Serialize)]
struct ModuleFunctionArg {
    pub help: Option<String>,
    pub r#type: Option<i32>,
    pub required: bool,
}

#[derive(serde::Serialize)]
struct ModuleFunction {
    pub description: Option<String>,
    pub args: IndexMap<String, ModuleFunctionArg>,
}

impl ModuleFunction {
    pub fn new(description: Option<String>) -> Self {
        Self {
            description: description,

            args: IndexMap::new(),
        }
    }
}

#[derive(serde::Serialize)]
struct Module {
    pub name: String,
    pub description: Option<String>,

    pub functions: IndexMap<String, ModuleFunction>
}

impl Module {
    pub fn new(name: &str, description: Option<String>) -> Self {
        Self {
            name: name.to_owned(),
            description: description,
            functions: IndexMap::new(),
        }
    }
}

fn yaml_from_comment(comment: &String) -> Option<serde_yaml::Value> {
    if comment.len() == 0 {
        return None;
    }

    match serde_yaml::from_str(&comment) {
        Ok(val) => Some(val),
        Err(err) => {
            eprintln!("Found string but failed to deserialize:\n{err}");
            None
        }
    }
}

fn get_location_info_from_path<'a>(path: &Vec<i32>, source_code_info: &'a SourceCodeInfo) -> Option<&'a Location> {
    for loc in source_code_info.location.iter() {
        if path == &loc.path {
            return Some(loc);
        }
    }

    None
}

fn combine_comments(location_info: &Location) -> String {
    let mut comment = String::new();

    for c in location_info.leading_detached_comments.iter() {
        comment += c;
    }

    comment += location_info.leading_comments();

    comment
}

fn main() {
    let mut input = Vec::new();
    std::io::stdin().read_to_end(&mut input).expect("input should be provided");

    let request = CodeGeneratorRequest::decode(input.as_slice()).expect("input should be valid CodeGeneratorRequest");

    eprintln!("request: {request:#?}");

    let file = &request.proto_file[0];

    let Some(source_code_info) = &file.source_code_info else {
        eprintln!("No source code info file, unable to generate manifest");
        return;
    };

    let description = if let Some(location_info) = get_location_info_from_path(&vec![2], &source_code_info) {
        let comment = combine_comments(&location_info);
        
        if let Some(yaml) = yaml_from_comment(&comment) {
            yaml.get("description")
                .and_then(|val| val.as_str())
                .and_then(|s| Some(s.to_owned()))
        } else {
            None
        }
    } else {
        None
    };

    let mut module = Module::new(file.package(), description);

    for (i, message_type) in file.message_type.iter().enumerate() {
        if let Some(name) = &message_type.name {
            let func_name = name.to_lowercase();
            if func_name.ends_with("request") {
                let path = vec![4i32, i as i32];
                if let Some(location_info) = get_location_info_from_path(&path, source_code_info) {
                    let comment = combine_comments(location_info);

                    if let Some(yaml) = yaml_from_comment(&comment) {
                        let description = yaml.get("description")
                            .and_then(|val| val.as_str())
                            .and_then(|s| Some(s.to_owned()));

                        let func_name = func_name.trim_end_matches("request");
                        let mut module_function = ModuleFunction::new(description);

                        for (j, field) in message_type.field.iter().enumerate() {
                            let path = vec![4i32, i as i32, 2i32, j as i32];

                            let Some(name) = &field.name else {
                                panic!("args must have name");
                            };

                            let Some(r#type) = field.r#type else {
                                panic!("args must have a type");
                            };

                            let mut module_func_arg = ModuleFunctionArg{
                                r#type: Some(r#type),
                                required: false,
                                help: None,
                            };

                            if let Some(location_info) = get_location_info_from_path(&path, source_code_info) {
                                let comment = combine_comments(location_info);

                                if let Some(yaml) = yaml_from_comment(&comment) {
                                    let help = yaml.get("help")
                                        .and_then(|val| val.as_str())
                                        .and_then(|s| Some(s.to_owned()));

                                    module_func_arg.help = help;

                                    let required = yaml.get("required")
                                        .and_then(|val| val.as_bool())
                                        .and_then(|s| Some(s.to_owned()))
                                        .unwrap_or(false);

                                    module_func_arg.required = required;
                                }
                            }

                            module_function.args.insert(name.to_owned(), module_func_arg);
                        }

                        module.functions.insert(func_name.to_owned(), module_function);
                    }
                }
            }
        }
    }


    let encoded = serde_yaml::to_string(&module).unwrap();

    eprintln!("encoded:\n{encoded}");

}
