extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{ToTokens, TokenStreamExt, format_ident, quote};
use syn::ItemImpl;
#[allow(unused_imports)]
use syn::{ItemFn, ItemTrait, parse_macro_input};

mod manager;
use manager::{ImplManager, MethodInfo};

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

    match syn::parse::<ItemImpl>(trait_input) {
        Ok(it) => {
            // dbg!(&it);
            let mut rpc_tokens = TokenStream::new();

            rpc_tokens.extend(tokens);

            let private_mod = emit_dispatch_private_mod(&it);
            rpc_tokens.extend(private_mod);

            let stub_definitions = emit_stub_definitions(&it);
            rpc_tokens.extend(stub_definitions);

            let dispatch_definition = emit_dispatch_definition(&it);
            rpc_tokens.extend(dispatch_definition);

            println!("rpc_tokens: {}", rpc_tokens);
            rpc_tokens
        }
        Err(err) => {
            println!("Failed to parse ItemTrait: {}", err);
            tokens
        }
    }
}

fn emit_stub_definition(dispatch_ident: &syn::Ident, mi: &MethodInfo) -> TokenStream {
    let MethodInfo(fn_name, args, ret_type) = mi;

    let stub_ident = format_ident!("__stub_{}_{}", dispatch_ident, fn_name);

    let arg_names: Vec<syn::Ident> = args.iter().map(|(n, _)| n.clone()).collect();

    let args_decode = if !args.is_empty() {
        quote! {
            let (#(#arg_names),*) = rmp_serde::decode::from_slice(args.as_slice()).unwrap();
        }
    } else {
        TokenStream::new().into()
    };

    let func_call = if ret_type.is_some() {
        quote! {
            let ret = #dispatch_ident::#fn_name(#(#arg_names),*);
            rmp_serde::encode::to_vec(&ret).unwrap()
        }
    } else {
        quote! {
            #dispatch_ident::#fn_name(#(#arg_names),*);
            std::vec::Vec::new()
        }
    };

    let stub_ident_str = stub_ident.to_string();

    let expanded = quote! {
        #[allow(non_snake_case)]
        fn #stub_ident(args: &std::vec::Vec<u8>) -> std::vec::Vec<u8> {
            println!("in {}", #stub_ident_str);
            #args_decode

            #func_call
        }
    };

    expanded.into()
}

fn emit_dispatch_definition(i: &ItemImpl) -> TokenStream {
    let manager = ImplManager::new(i.clone());
    let ident = manager.get_ident().unwrap();
    let Ok(mi) = manager.get_methods() else {
        panic!("oh no");
    };

    let mut ids: Vec<usize> = Vec::new();
    let mut calls: Vec<syn::Ident> = Vec::new();

    for (i, method_info) in mi.iter().enumerate() {
        let stub_ident = format_ident!("__stub_{}_{}", ident, method_info.0);
        ids.push(i);
        calls.push(stub_ident)
    }

    let expanded = quote! {
        #[unsafe(no_mangle)]
        pub extern "C" fn dispatch(id: usize, arg_ptr: *mut u8, arg_len: usize) -> *mut std::ffi::c_void {
            let args = unsafe { std::vec::Vec::from_raw_parts(arg_ptr, arg_len, arg_len) };
            println!("in dispatch, id: {} arg_ptr: {} arg_len: {}", id, arg_ptr as usize, arg_len);
            let ret = match id {
                #(#ids => #calls(&args)),*,
                _ => std::vec::Vec::new(),
            };

            std::mem::forget(args);
            let encoded = rmp_serde::encode::to_vec(&(ret.len(), ret)).unwrap();
            let slice = encoded.into_boxed_slice();
            let ptr = std::boxed::Box::leak(slice);
            ptr.as_mut_ptr() as *mut std::ffi::c_void
        }
    };

    println!("emit_stub_definition expanded: {expanded}");
    expanded.into()
}

fn emit_stub_definitions(i: &ItemImpl) -> TokenStream {
    let manager = ImplManager::new(i.clone());
    let ident = manager.get_ident().unwrap();

    let mut func_calls = TokenStream::new();

    if let Ok(methods) = manager.get_methods() {
        for method in methods {
            println!("MethodInfo: {:#?}", method);
            let func_call = emit_stub_definition(&ident, &method);
            func_calls.extend(func_call);
        }
    } else {
        println!("ImplManager::get_methods FAILED");
    }

    func_calls
}

fn emit_dispatch_private_mod(i: &ItemImpl) -> TokenStream {
    let manager = ImplManager::new(i.clone());
    let mod_name = if let Some(ident) = manager.get_ident() {
        format_ident!("__{}_mod", ident)
    } else {
        panic!("could not get_ident from impl: {:#?}", &i);
    };

    let expanded = quote! {
        #[allow(non_snake_case)]
        pub mod #mod_name {

            type StubFn = fn(std::vec::Vec<u8>) -> std::vec::Vec<u8>;

            fn init_dispatch_map() -> std::vec::Vec<StubFn> {
                    std::vec![]
            }

            pub fn get_dispatch_map()
            -> &'static std::sync::Mutex<std::vec::Vec<StubFn>> {
                static DISPATCH_MAP: std::sync::OnceLock<
                std::sync::Mutex<std::vec::Vec<StubFn>>
                > = std::sync::OnceLock::new();
                DISPATCH_MAP.get_or_init(|| std::sync::Mutex::new(init_dispatch_map()))
            }

            pub enum DispatchError {
                Success = 0,
                UnknownFunction = 1,
                MapMutexFailure = 2,
            }
        }
    };

    expanded.into()
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
