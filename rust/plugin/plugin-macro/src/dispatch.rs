use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{FnArg, ImplItem, ItemImpl, PathArguments, ReturnType, Signature, Type, parse_macro_input};

const PLUGIN_START: &str = "call_";

pub(crate) fn _plugin_dispatch(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemImpl);
    let self_ty = &input.self_ty;

    let mut methods = Vec::new();

    for ele in &input.items {
        if let ImplItem::Fn(method) = ele {
            let name = method.sig.ident.to_string();
            if name.starts_with(PLUGIN_START) // 方法以call_开头
                && method.sig.asyncness.is_some() // 方法是异步的
                && let Some(FnArg::Receiver(recv)) = method.sig.inputs.first() 
                && recv.reference.is_some() // 方法的第一个参数是self
                && recv.mutability.is_none() // 但不是mut self
            {
                methods.push(method);
            }
        }
    }

    let mut match_arms = Vec::new();

    for method in methods {
        let method_name = method.sig.ident.to_string();
        let method_ident = &method.sig.ident;

        let params:Vec<_> = method.sig.inputs.iter().skip(1).collect();
        let params_len = params.len();
        let (param_extraction,call) = if params_len==0 {
            (quote! {},
            
             quote! { self.#method_ident().await })
        } else if params_len == 1 {
            let arg_ty = match params[0] {
                FnArg::Typed(pat_type) => &pat_type.ty,
                _ => return newerror(params[0], "expected typed parameter")
            };
            (quote! {
                let arg: #arg_ty = ::plugin::from_value(params.clone())?;
            },
            quote! { self.#method_ident(arg).await })
        } else {
            let tuple_types = params.iter().map(|arg| {
                 match arg {
                    FnArg::Typed(pat_type) => &pat_type.ty,
                    _ => panic!("expected typed parameter")
                }
            });
            let tuple_tys = quote! { (#(#tuple_types),*) };
            let tuple_vars = (0..params_len).map(|i| format_ident!("arg{i}") );
            let tuple_vars2 = (0..params_len).map(|i| format_ident!("arg{i}") );
            (quote! {
                let (#(#tuple_vars,)*): #tuple_tys = ::plugin::from_value(params.clone())?; 
            },
            quote! { self.#method_ident(#(#tuple_vars2),*).await })
        };

        let is_result = match is_result(&method.sig) {
            Ok(r) => r,
            Err(e) => return e.to_compile_error().into(),
        };
        let return_conversion = if is_result {
            quote! {
                let result = #call?;
                Ok(::plugin::to_value(result)?)
            }
        } else {
            quote! {
                let result = #call;
                Ok(::plugin::to_value(&result)?)
            }
        };

        match_arms.push(quote! {
            #method_name => {
                #param_extraction
                #return_conversion
            }
        });
    }

    let plugin_impl = quote! {
        #[::plugin::async_trait]
        impl ::plugin::Plugin for #self_ty {
            async fn call(&self, input: ::plugin::Value) -> ::std::result::Result<::plugin::Value, Box<dyn std::error::Error + Send + Sync>> {
                let err = |str: &str| Box::<dyn std::error::Error + Send + Sync>::from(str);
                let (method, params) = match input {
                    ::plugin::Value::Object(mut map) => {
                        let method = map.remove("method").ok_or_else(|| err("no method"))?;
                        let params = map.remove("params").ok_or_else(|| err("no params"))?;
                        (method, params)
                    }
                    _ => return Err(err("input mut be object with method and params")),
                };
                match method.as_str().ok_or_else(|| err("method not string"))? {
                    #(#match_arms,)*
                    _ => Err(err(format!("unknown method: {}", method).as_str())),
                }
            }
        }
    };

   quote! {
        #input
        #plugin_impl
    }.into()
}

fn newerror<T: quote::ToTokens, U: std::fmt::Display>(tokens:T,message:U)->TokenStream{
    syn::Error::new_spanned(tokens, message).to_compile_error().into()
}


fn is_result(sig: &Signature) -> Result<bool, syn::Error> {
    // 判断返回类型是否为 Result（启发式：类型路径最后一段为 "Result"）
    if let ReturnType::Type(_, ty) = &sig.output
        && let Type::Path(type_path) = &**ty
    {
        let last_seg = type_path
            .path
            .segments
            .last()
            .ok_or(syn::Error::new_spanned(
                sig,
                "no last segment in return type path".to_string(),
            ))?;
        if last_seg.ident == "Result" {
            // 检查泛型参数数量，如果只有一个，很可能是用户定义的单参数别名
            if let PathArguments::AngleBracketed(args) = &last_seg.arguments
                && args.args.len() == 1
            {
                return Err(syn::Error::new_spanned(
                    ty,
                    "use `std::result::Result` instead or use `type MyResult<T, E = MyErr> = Result<T, E>` instead".to_string(),
                ));
            }
            return Ok(true);
        }
    }
    Ok(false)
}