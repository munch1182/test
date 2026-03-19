use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    FnArg, Ident, ItemFn, Pat, PatType, PathArguments, ReturnType, Signature, Type,
    parse_macro_input,
};

// 定义包装函数后缀常量
const WRAPPER_SUFFIX: &str = "_generate";
const NO_INPUT_PARAM: &str = "WindowState";

/// 属性宏：将函数转换为 IPC 可调用的包装函数，并生成同名模块。
///
/// 原函数保持不变，生成的包装函数位于与原函数同名的模块内，可通过 `模块名::_原函数名_generate` 调用。
///
/// 返回值可以为 `serde_json::Value`，也可以是 `std::result::Result<serde_json::Value, Box<dyn std::error::Error>>`;
/// 如果返回值不是 `Result`, 要保证当前简写的 `Result` 能指向 `std::result::Result`;
///
/// 支持宿主状态参数：如果原函数最后一个参数类型为 `WindowState<H>`，则将其视为宿主状态，不会出现在参数结构体中，
/// 并在生成的包装函数中通过第二个参数传入。参数模式可以是 `state: WindowState<H>` 或 `WindowState(state): WindowState<H>`。
///
/// # 用法
/// ```rust
/// #[bridge::fun]
/// pub fn add(a: i32, b: i32, state: WindowState<MyState>) -> std::result::Result<i32> {
///     // 可以使用 state 调用宿主方法
///     Ok(a + b)
/// }
///
/// #[bridge::fun]
/// pub fn list_plugins(WindowState(state): WindowState<AppState>) -> Vec<Plugin> {
///     // 解构方式同样有效，state 是内部 Arc<AppState>
///     vec![]
/// }
/// ```
///
/// 生成的代码：
/// ```rust
/// // 原函数
/// pub fn add(a: i32, b: i32, state: WindowState<MyState>) -> std::result::Result<i32> { ... }
///
/// // 同名模块
/// pub mod add {
///     use super::*;
///
///     #[derive(serde::Deserialize)]
///     struct _AddArgs { a: i32, b: i32 }
///
///     pub fn _add_generate(
///         _arg: Option<serde_json::Value>,
///         state: WindowState<MyState>,
///     ) -> Pin<Box<dyn Future<Output = std::result::Result<serde_json::Value, Box<dyn std::error::Error>>> + Send>> {
///         Box::pin(async move {
///             let _arg = _arg.ok_or_else(|| Box::<dyn std::error::Error>::from("need args but got none"))?;
///             let args: _AddArgs = serde_json::from_value(_arg)?;
///             let a = args.a;
///             let b = args.b;
///             let result = super::add(a, b, state).await?;
///             Ok(serde_json::json!(result))
///         })
///     }
/// }
///
/// // 无参数函数类似，但不生成结构体
/// ```
#[proc_macro_attribute]
pub fn bridge(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = input;

    let fn_name = &sig.ident;
    let fn_name_str = fn_name.to_string();
    let wrapper_name = Ident::new(
        &format!("_{}{}", fn_name_str, WRAPPER_SUFFIX),
        fn_name.span(),
    );

    // 提取普通参数和宿主状态参数
    let mut arg_names = Vec::new();
    let mut arg_types = Vec::new();
    let mut has_host = false; // 标记原函数是否接受宿主状态
    let mut host_ty = None; // 保存宿主状态的具体类型（如果有）

    for (idx, arg) in sig.inputs.iter().enumerate() {
        match arg {
            FnArg::Typed(PatType { pat, ty, .. }) => {
                // 检查是否为宿主状态参数：必须是最后一个参数，且类型名为 `WindowState`
                let is_host_candidate = idx == sig.inputs.len() - 1;
                if is_host_candidate && let Type::Path(type_path) = &**ty {
                    let last_seg = type_path.path.segments.last().unwrap();
                    if last_seg.ident == NO_INPUT_PARAM {
                        has_host = true;
                        host_ty = Some(ty.clone());
                        continue; // 不加入普通参数列表
                    }
                }

                // 不是宿主状态参数，需要提取变量名（只支持简单标识符模式）
                match &**pat {
                    Pat::Ident(pat_ident) => {
                        let name = pat_ident.ident.clone();
                        arg_names.push(name);
                        arg_types.push(ty.clone());
                    }
                    Pat::TupleStruct(_) => {
                        // 如果它是最后一个参数且为 WindowState，我们已经在上面的 is_host_candidate 中处理了
                        // 如果走到这里，说明不是最后一个参数，或者类型不是 WindowState，报错
                        return syn::Error::new_spanned(
                            pat,
                            "tuple struct pattern is only allowed for the last parameter of type WindowState",
                        )
                        .to_compile_error()
                        .into();
                    }
                    _ => {
                        return syn::Error::new_spanned(
                            pat,
                            "only support simple parameter names like `name: Type`",
                        )
                        .to_compile_error()
                        .into();
                    }
                }
            }
            FnArg::Receiver(_) => {
                return syn::Error::new_spanned(arg, "unsupport self")
                    .to_compile_error()
                    .into();
            }
        }
    }

    let is_async = sig.asyncness.is_some();
    let has_params = !arg_names.is_empty();

    // 判断返回类型是否为 Result
    let is_result = match is_result(&sig) {
        Ok(e) => e,
        Err(s) => return s.to_compile_error().into(),
    };

    // 根据是否有参数和宿主状态生成不同的包装函数体
    let wrapper_body = if has_params {
        let struct_name = Ident::new(
            &format!("_{}Args", fn_name_str.to_case(Case::Pascal)),
            fn_name.span(),
        );
        let arg_struct_fields = arg_names.iter().zip(arg_types.iter()).map(|(name, ty)| {
            quote! { #name: #ty }
        });
        let arg_assignments = arg_names.iter().map(|name| {
            quote! { let #name = args.#name; }
        });

        // 构造调用原函数的表达式（通过 super:: 路径）
        let call_expr = if is_async {
            if has_host {
                quote! { super::#fn_name(#(#arg_names,)* state).await }
            } else {
                quote! { super::#fn_name(#(#arg_names),*).await }
            }
        } else if has_host {
            quote! { super::#fn_name(#(#arg_names,)* state) }
        } else {
            quote! { super::#fn_name(#(#arg_names),*) }
        };

        // 根据是否返回 Result 生成不同的转换代码
        let convert_code = if is_result {
            quote! {
                let result = #call_expr?;
                serde_json::to_value(result).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
            }
        } else {
            quote! {
                let result = #call_expr;
                serde_json::to_value(&result).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
            }
        };

        // 生成带参数结构体的完整代码（结构体设为私有）
        if has_host {
            let host_ty = host_ty.unwrap();
            quote! {
                // 参数结构体（私有）
                #[derive(serde::Deserialize)]
                struct #struct_name {
                    #(#arg_struct_fields,)*
                }

                // 包装函数（使用具体宿主类型）
                pub fn #wrapper_name(
                    _arg: Option<serde_json::Value>,
                    state: #host_ty,
                ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<serde_json::Value, Box<dyn std::error::Error>>> + Send>> {
                    Box::pin(async move {
                        let _arg = _arg.ok_or_else(|| Box::<dyn std::error::Error>::from("need args but got none"))?;
                        let args: #struct_name = serde_json::from_value(_arg)?;
                        #(#arg_assignments)*
                        #convert_code
                    })
                }
            }
        } else {
            // 没有宿主状态，使用泛型 H 和 ::window::WindowState<H>
            quote! {
                // 参数结构体（私有）
                #[derive(serde::Deserialize)]
                struct #struct_name {
                    #(#arg_struct_fields,)*
                }

                // 包装函数（泛型宿主状态，但忽略它）
                pub fn #wrapper_name<H>(
                    _arg: Option<serde_json::Value>,
                    _state: ::window::WindowState<H>,
                ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<serde_json::Value, Box<dyn std::error::Error>>> + Send>> {
                    Box::pin(async move {
                        let _arg = _arg.ok_or_else(|| Box::<dyn std::error::Error>::from("need args but got none"))?;
                        let args: #struct_name = serde_json::from_value(_arg)?;
                        #(#arg_assignments)*
                        #convert_code
                    })
                }
            }
        }
    } else {
        // 无参数：直接调用函数，忽略 _arg
        let call_expr = if is_async {
            if has_host {
                quote! { super::#fn_name(state).await }
            } else {
                quote! { super::#fn_name().await }
            }
        } else if has_host {
            quote! { super::#fn_name(state) }
        } else {
            quote! { super::#fn_name() }
        };

        let convert_code = if is_result {
            quote! {
                let result = #call_expr?;
                serde_json::to_value(result).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
            }
        } else {
            quote! {
                let result = #call_expr;
                serde_json::to_value(&result).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
            }
        };

        if has_host {
            let host_ty = host_ty.unwrap();
            quote! {
                // 包装函数（使用具体宿主类型）
                pub fn #wrapper_name(
                    _arg: Option<serde_json::Value>,
                    state: #host_ty,
                ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<serde_json::Value, Box<dyn std::error::Error>>> + Send>> {
                    Box::pin(async move {
                        #convert_code
                    })
                }
            }
        } else {
            quote! {
                // 包装函数（泛型宿主状态，但忽略它）
                pub fn #wrapper_name<H>(
                    _arg: Option<serde_json::Value>,
                    _state: ::window::WindowState<H>,
                ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<serde_json::Value, Box<dyn std::error::Error>>> + Send>> {
                    Box::pin(async move {
                        #convert_code
                    })
                }
            }
        }
    };

    // 将包装函数和结构体放入与原函数同名的模块中
    let module_body = quote! {
        use super::*;
        #wrapper_body
    };

    let expanded = quote! {
        // 保留原函数
        #(#attrs)* #vis #sig #block

        // 生成同名模块，可见性与原函数一致
        #vis mod #fn_name {
            #module_body
        }
    };

    TokenStream::from(expanded)
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
