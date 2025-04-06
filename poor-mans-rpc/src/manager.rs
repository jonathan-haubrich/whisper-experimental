use std::ops::{self, Deref};

use syn::{Ident, ImplItem, ItemImpl};

pub(crate) struct ImplManager {
    parsed: ItemImpl,
}

#[derive(Debug)]
pub(crate) struct MethodInfo(Ident, Vec<(Ident, Ident)>, Option<Ident>);

pub(crate) struct InvalidImplItemError;
impl TryFrom<&ImplItem> for MethodInfo {
    type Error = InvalidImplItemError;
    fn try_from(value: &ImplItem) -> Result<Self, Self::Error> {
        if let ImplItem::Fn(item_fn) = value {
            let fn_name = item_fn.sig.ident.clone();
            println!("got fn_name: {fn_name}");

            let mut args = Vec::new();

            // dbg!(&item_fn.sig.inputs);

            for arg in &item_fn.sig.inputs {
                // dbg!(arg);
                if let syn::FnArg::Typed(pt) = arg {
                    let arg_ident;
                    let arg_type;
                    if let syn::Pat::Ident(ident) = *pt.pat.clone() {
                        arg_ident = ident.ident.clone();
                    } else {
                        return Err(InvalidImplItemError {});
                    }

                    if let syn::Type::Path(p) = *pt.ty.clone() {
                        if let Some(s) = p.path.segments.get(0) {
                            arg_type = s.ident.clone();
                        } else {
                            return Err(InvalidImplItemError {});
                        }
                    } else {
                        return Err(InvalidImplItemError {});
                    }

                    args.push((arg_ident, arg_type));
                }
            }

            // dbg!(&item_fn.sig.output);
            match item_fn.sig.output.clone() {
                syn::ReturnType::Default => {
                    println!("Default return type");
                    let method_info = MethodInfo(fn_name, args, None);
                    println!("successfully parsed method info: {:#?}", method_info);

                    Ok(method_info)
                }
                syn::ReturnType::Type(_, t) => {
                    if let syn::Type::Path(p) = *t.clone() {
                        if let Some(s) = p.path.segments.get(0) {
                            let rt_name = s.ident.clone();

                            println!("got rt_name: {rt_name}");

                            let method_info = MethodInfo(fn_name, args, Some(rt_name));

                            println!("successfully parsed method info: {:#?}", method_info);

                            Ok(method_info)
                        } else {
                            Err(InvalidImplItemError {})
                        }
                    } else {
                        Err(InvalidImplItemError {})
                    }
                }
            }
        } else {
            Err(InvalidImplItemError {})
        }
    }
}

impl ImplManager {
    pub(crate) fn new(parsed: ItemImpl) -> ImplManager {
        ImplManager {
            parsed: parsed.clone(),
        }
    }

    pub(crate) fn get_ident(&self) -> Option<Ident> {
        match *self.parsed.self_ty.clone() {
            syn::Type::Path(path) => {
                if path.path.segments.is_empty() {
                    return None;
                }
                Some(path.path.segments[0].ident.clone())
            }
            _ => None,
        }
    }

    pub(crate) fn get_methods(&self) -> Result<Vec<MethodInfo>, InvalidImplItemError> {
        // for each function we need:
        //   1. function name to create __stub_function(...)
        //   2. func args: name and type for __stub_function_ArgsStruct
        //   3. func return type? (maybe nor)

        let mut methods = Vec::new();
        for item in self.parsed.items.iter() {
            if let Ok(method_info) = MethodInfo::try_from(item) {
                methods.push(method_info);
            } else {
                continue;
            }
        }

        Ok(methods)
    }
}
