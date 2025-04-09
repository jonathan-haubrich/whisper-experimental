use syn::{Ident, ImplItem, ItemImpl};

pub(crate) struct ImplManager {
    parsed: ItemImpl,
}

#[derive(Debug)]
pub(crate) struct MethodInfo(pub Ident, pub Vec<(Ident, Ident)>, pub Option<Ident>);

pub(crate) struct InvalidImplItemError;
impl TryFrom<&ImplItem> for MethodInfo {
    type Error = InvalidImplItemError;
    fn try_from(value: &ImplItem) -> Result<Self, Self::Error> {
        if let ImplItem::Fn(item_fn) = value {
            let fn_name = item_fn.sig.ident.clone();

            let mut args = Vec::new();

            // dbg!(&item_fn.sig.inputs);

            for arg in &item_fn.sig.inputs {
                dbg!(arg);
                let syn::FnArg::Typed(pt) = arg else {
                    return Err(InvalidImplItemError {});
                };

                match (*pt.pat.clone(), *pt.ty.clone()) {
                    (syn::Pat::Ident(ident), syn::Type::Path(path)) => {
                        args.push((ident.ident.clone(), path.path.segments[0].ident.clone()))
                    }
                    _ => return Err(InvalidImplItemError {}),
                }
            }

            // dbg!(&item_fn.sig.output);
            match &item_fn.sig.output {
                syn::ReturnType::Default => {
                    let method_info = MethodInfo(fn_name, args, None);

                    Ok(method_info)
                }
                syn::ReturnType::Type(_, t) => {
                    if let syn::Type::Path(p) = *t.clone() {
                        if let Some(s) = p.path.segments.get(0) {
                            let rt_name = s.ident.clone();

                            let method_info = MethodInfo(fn_name, args, Some(rt_name));

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
