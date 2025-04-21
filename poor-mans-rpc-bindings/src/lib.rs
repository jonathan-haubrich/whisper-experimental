use syn::ItemImpl;

pub trait BindingGenerator {
    fn generate(dispatch_impl: ItemImpl);
}

pub struct PythonBindings {}

impl BindingGenerator for PythonBindings {
    fn generate(dispatch_impl: ItemImpl) {
        dbg!(dispatch_impl);
    }
}