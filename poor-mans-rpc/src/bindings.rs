use syn::ItemImpl;

pub trait BindingGenerator {
    fn gen_bindings(dispatch_impl: ItemImpl);
}

pub struct PythonBindingGenerator {}

impl BindingGenerator for PythonBindings {
    fn gen_bindings(dispatch_impl: ItemImpl) {
        dbg!(dispatch_impl);
    }
}