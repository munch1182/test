use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Ident};

pub(crate)  fn _derive_value(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;

    // 只处理结构体，且必须为命名字段
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("Value derive only supports structs with named fields"),
        },
        _ => panic!("Value derive only supports structs"),
    };

    // 收集需要转换的字段（跳过标记了 `#[value(skip)]` 的字段）
    let field_infos: Vec<_> = fields
        .iter()
        .filter_map(|f| {
            // 检查是否有 `#[value(skip)]` 属性
            let skip = f.attrs.iter().any(|attr| {
                attr.path()
                    .is_ident("value")
                    && attr
                        .parse_args::<Ident>()
                        .map(|ident| ident == "skip")
                        .unwrap_or(false)
            });
            if skip {
                None
            } else {
                // 获取字段名和类型
                let name = f.ident.as_ref().unwrap();
                let ty = &f.ty;
                Some((name, ty))
            }
        })
        .collect();

    // 生成 `From` 实现
    let from_impl = {
        let field_names: Vec<_> = field_infos.iter().map(|(name, _)| name).collect();
        let field_keys: Vec<String> = field_names.iter().map(|name| name.to_string()).collect();

        quote! {
            impl #generics From<#name #generics> for Value {
                fn from(val: #name #generics) -> Self {
                    let mut map = std::collections::HashMap::new();
                    #(
                        map.insert(#field_keys.to_string(), Value::from(val.#field_names));
                    )*
                    Value::Map(map)
                }
            }
        }
    };

    // 生成 `TryFrom<Value>` 实现（消耗所有权）
    let try_from_owned_impl = {
        let field_names: Vec<_> = field_infos.iter().map(|(name, _)| name).collect();
        let field_types: Vec<_> = field_infos.iter().map(|(_, ty)| ty).collect();
        let field_keys: Vec<String> = field_names.iter().map(|name| name.to_string()).collect();

        quote! {
            impl #generics TryFrom<Value> for #name #generics {
                type Error = plugin::ValueParseError;

                fn try_from(value: Value) -> std::result::Result<Self, Self::Error> {
                    match value {
                        Value::Map(mut map) => {
                            Ok(#name {
                                #(
                                    #field_names: {
                                        let field_value = map.remove(#field_keys).ok_or(plugin::ValueParseError)?;
                                        <#field_types>::try_from(field_value)?
                                    },
                                )*
                            })
                        }
                        _ => Err(plugin::ValueParseError),
                    }
                }
            }
        }
    };

    // 生成 `TryFrom<&Value>` 实现（通过引用，克隆字段值）
    let try_from_ref_impl = {
        let field_names: Vec<_> = field_infos.iter().map(|(name, _)| name).collect();
        let field_types: Vec<_> = field_infos.iter().map(|(_, ty)| ty).collect();
        let field_keys: Vec<String> = field_names.iter().map(|name| name.to_string()).collect();

        quote! {
            impl #generics TryFrom<&Value> for #name #generics {
                type Error = plugin::ValueParseError;

                fn try_from(value: &Value) -> std::result::Result<Self, Self::Error> {
                    match value {
                        Value::Map(map) => {
                            Ok(#name {
                                #(
                                    #field_names: {
                                        let field_value = map.get(#field_keys)
                                            .ok_or(plugin::ValueParseError)?;
                                        // 克隆字段值再转换，避免消耗原值
                                        <#field_types>::try_from(field_value.clone())?
                                    },
                                )*
                            })
                        }
                        _ => Err(plugin::ValueParseError),
                    }
                }
            }
        }
    };

    // 合并生成代码
    let expanded = quote! {
        #from_impl
        #try_from_owned_impl
        #try_from_ref_impl
    };

    TokenStream::from(expanded)
}