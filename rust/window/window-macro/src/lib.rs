use proc_macro::TokenStream;
use quote::quote;
use syn::{
    FnArg, Ident, ItemFn, PatType, PathArguments, ReturnType, Signature, Type, parse_macro_input,
};

paste::paste!(
    const [<WRPPER_NAME>]: &str = "_generate";
);

/// 属性宏：将函数转换为 IPC 可调用的包装函数
///
/// 返回值可以为serde_json::Value，也可以是std::result::Result<serde_json::Value, Box<dyn std::error::Error>>;
///
/// # 用法
/// ```rust
/// #[bridge::fun]
/// pub fn add(a: i32, b: i32) -> std::result::Result<i32> {
///     Ok(a + b)
/// }
/// ```
///
/// 生成的代码：
/// ```rust
/// #[derive(serde::Deserialize)]
/// struct _AddArgs { a: i32, b: i32 }
///
/// pub fn _add_generate(
///     _arg: serde_json::Value,
/// ) -> Pin<Box<dyn Future<Output = std::result::Result<serde_json::Value, Box<dyn std::error::Error>>> + Send>> {
///     Box::pin(async move {
///         let args: _AddArgs = serde_json::from_value(_arg)?;
///         let result = add(args.a, args.b)?;
///         Ok(serde_json::json!(result))
///     })
/// }
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
    let struct_name = Ident::new(
        &format!("_{}Args", fn_name_str.to_uppercase()),
        fn_name.span(),
    );
    let wrapper_name = Ident::new(
        &format!("_{}{}", fn_name_str, paste::paste!([<WRPPER_NAME>])),
        fn_name.span(),
    );

    // 提取参数名和类型
    let mut arg_names = Vec::new();
    let mut arg_types = Vec::new();
    for arg in sig.inputs.iter() {
        match arg {
            FnArg::Typed(PatType { pat, ty, .. }) => {
                if let syn::Pat::Ident(pat_ident) = &**pat {
                    arg_names.push(pat_ident.ident.clone());
                    arg_types.push(ty.clone());
                } else {
                    return syn::Error::new_spanned(
                        pat,
                        "only support simple param, like `name: Type`",
                    )
                    .to_compile_error()
                    .into();
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

    // 构造调用原函数的表达式
    let call_expr = if is_async {
        quote! { #fn_name(#(#arg_names),*).await }
    } else {
        quote! { #fn_name(#(#arg_names),*) }
    };

    // 判断返回类型是否为 Result（启发式：类型路径最后一段为 "Result"）
    let is_result = match is_result(&sig) {
        Ok(e) => e,
        Err(s) => return s.to_compile_error().into(),
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

    let expanded = quote! {
        // 保留原函数
        #(#attrs)* #vis #sig #block

        // 参数结构体
        #[derive(serde::Deserialize)]
        #vis struct #struct_name {
            #(#arg_names: #arg_types,)*
        }

        // 包装函数
        #vis fn #wrapper_name(
            _arg: serde_json::Value,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<serde_json::Value, Box<dyn std::error::Error>>> + Send>> {
            Box::pin(async move {
                let args: #struct_name = serde_json::from_value(_arg)?;
                #(let #arg_names = args.#arg_names;)*
                #convert_code
            })
        }
    };

    TokenStream::from(expanded)
}

fn is_result(sig: &Signature) -> Result<bool, syn::Error> {
    // 判断返回类型是否为 Result（启发式：类型路径最后一段为 "Result"）
    if let ReturnType::Type(_, ty) = &sig.output
        && let Type::Path(type_path) = &**ty
    {
        let last_seg = type_path.path.segments.last().unwrap();
        if last_seg.ident == "Result" {
            // 检查泛型参数数量，如果只有一个，很可能是用户定义的单参数别名
            if let PathArguments::AngleBracketed(args) = &last_seg.arguments
                && args.args.len() == 1
            {
                return Err(syn::Error::new_spanned(
                    ty,
                    "use std::result::Result instead".to_string(),
                ));
            }
            return Ok(true);
        }
    }
    Ok(false)
}
