use pmr_bindings::{self, BindingGenerator, PythonBindings};
use syn::Item;

fn main() {
    let content = std::fs::read_to_string("src/lib.rs").unwrap();
    let file = syn::parse_file(&content).unwrap();
    
    for item in file.items {
        match item {
            Item::Impl(item_impl) => {
                PythonBindings::generate(item_impl);
            },
            _ => continue,
        }
    }

    //pmr_bindings::PythonBindings::gen_bindings();
}