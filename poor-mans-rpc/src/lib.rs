extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{ToTokens, format_ident, quote};
#[allow(unused_imports)]
use syn::{ItemFn, ItemTrait, parse_macro_input};

fn emit_procedure_stub(item_fn: &ItemFn) -> TokenStream {
    let stub_name = format_ident!("__stub_{}", item_fn.sig.ident);
    let stub_args_type_name = format_ident!("{}_args", stub_name);

    let mut fields = Vec::new();

    for arg in item_fn.sig.inputs.iter() {
        match arg {
            syn::FnArg::Typed(t) => {
                let field_name = match *t.pat.clone() {
                    syn::Pat::Ident(i) => i.ident,
                    _ => unreachable!(),
                };
                let field_type = match *t.ty.clone() {
                    syn::Type::Path(p) => p.path.segments[0].ident.clone(),
                    _ => unreachable!(),
                };
                let field = quote! { #field_name: #field_type, };
                fields.push(field)
            }
            syn::FnArg::Receiver(_) => unreachable!(),
        }
    }

    let expanded = quote! {
        #[unsafe(no_mangle)]
        pub extern "C" fn #stub_name(args: Vec<u8>) -> Vec<u8> {
            #[allow(non_camel_case_types)]
            struct #stub_args_type_name{
                #( #fields )*
            }

            Vec::<u8>::new()
        }

    };

    let expanded = TokenStream::from(expanded);
    // println!("{}", expanded);
    expanded
}

/// Dispatcher is used to decorate a trait impl
/// The trait implementation defines the functions that are available through
/// the exported `dispatch` function.
///
/// With a trait impl like:
/// ```
/// impl Dispatcher for Dispatch {
///     fn func(arg1: String) -> String {
///         ()
///     }
/// }
/// ```
///
/// The generated code for dispatching the function would look like:
/// ```
///
/// fn __stub_dispatch_func(args: Vec<u8>) -> Vec<u8> {
///
///     let (arg1) = serde_rmp::decode::from_slice(args.as_slice()).unwrap();
///
///     let ret = Dispatch::func1(arg1);
///
///     seder_rmp::encode::to_vec(ret).unwrap()
/// }
///
/// #[unsafe(no_mangle)]
/// pub extern "C" fn dispatch(id: String, args: Vec<u8>) -> Vec<u8> {
///     let dispatch_map = get_dispatch_map();
///
///     match dispatch_map.get(id) {
///         Ok(stub_fn) => stub_fn
///     }
///
///     match id {
///         "func1" => __stub_dispatch_func(args),
///         _ => unreachable!()
///     }
/// }
///
/// fn get_dispatch_map() -> &'static std::sync::Mutex<std::collections::HashMap<String, StubFn>> {
///     static DISPATCH_MAP: std::sync::OnceLock<
///         std::sync::Mutex<std::collections::HashMap<String, StubFn>>,
///     > = std::sync::OnceLock::new();
///     DISPATCH_MAP.get_or_init(|| std::sync::Mutex::new(init_dispatch_map()))
/// }
///
/// type StubFn = fn(Vec<u8>) -> Vec<u8>;
///
/// fn init_dispatch_map() -> std::collections::HashMap<String, StubFn> {
///     let mut dispatch_map: std::collections::HashMap<String, StubFn> =
///         std::collections::HashMap::new();
///     dispatch_map.insert(get_random_string(16), __stub_dispatch_func);
///
///     dispatch_map
/// }
///
/// #[unsafe(no_mangle)]
/// pub extern "C" fn init() -> Vec<u8> {
///     
/// }
/// ```
#[proc_macro_attribute]
pub fn dispatcher(_: TokenStream, tokens: TokenStream) -> TokenStream {
    let trait_input = tokens.clone();

    match syn::parse::<ItemTrait>(trait_input) {
        Ok(it) => {
            dbg!(it);
            tokens
        }
        _ => tokens,
    }
}

#[proc_macro_attribute]
pub fn procedure(_: TokenStream, item: TokenStream) -> TokenStream {
    // preserve token stream
    let original = item.clone();
    let fn_input = item.clone();
    //
    match syn::parse::<ItemFn>(fn_input) {
        Ok(f) => {
            // println!("function parsed successfully");
            // dbg!(&f);
            let mut procedure_stub = emit_procedure_stub(&f);
            procedure_stub.extend(item);
            // println!("{}", procedure_stub);
            procedure_stub
        }
        Err(err) => {
            println!("Failed to parse: {:#?}", err);
            item
        }
    }
}
