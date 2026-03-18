use libcommon::{ext::FileDirCreateExt, newerr, prelude::Result};
use std::{collections::HashMap, fs, path::Path};
use syn::{
    Attribute, FnArg, GenericArgument, ItemFn, Pat, PatType, PathArguments, Type,
    visit::{Visit, visit_item_fn},
};

///
/// 只指定文件中收集带有指定属性的方法, 生成对应的ts方法并输出到指定文件中
///
/// #参数
/// `files`: 要识别的文件列表
/// `attrs`: 要识别的属性列表
/// `output`: 输出文件路径
///
pub fn generate_ts(
    files: &[impl AsRef<Path>],
    attrs: &[&str],
    output: impl AsRef<Path>,
) -> Result<()> {
    let mut collector = FunCollector::new(attrs);

    for ele in files {
        let src = fs::read_to_string(ele)?;
        let ast = syn::parse_file(&src)?;
        collector.visit_file(&ast);
    }

    let funs = collector.funs;
    let ty_map = collector.ty_map;
    let mut output_str = String::new();

    output_str.push_str("// This file is auto-generated. Do not edit manually.\n\n");
    for (ty_name, fields) in &ty_map {
        output_str.push_str(&format!("export interface {ty_name} {{\n"));
        for (field_name, field_ty) in fields {
            output_str.push_str(&format!("    {field_name}: {};\n", field_ty.to_ts_string()));
        }
        output_str.push_str("}\n\n");
    }

    output_str.push_str(
        r#"
declare global {
  interface Window {
    bridge: {
      send<T>(command: string, payload: any | undefined): Promise<T>;
    };
  }
}
"#,
    );
    output_str.push_str("\n\n");

    // 生成对象开始
    output_str.push_str("export const commands = {\n");

    for fun in funs {
        let return_ts = fun.return_ty.to_ts_string();
        let fun_name = &fun.name;

        if fun.param.is_empty() {
            output_str.push_str(&format!(
                "    {}: (): Promise<{}> => window.bridge.send<{}>('{}', undefined),\n",
                fun_name, return_ts, return_ts, fun_name
            ));
        } else {
            let fields: Vec<String> = fun
                .param
                .iter()
                .map(|(name, ty)| format!("{}: {}", name, ty.to_ts_string()))
                .collect();
            let arg_type = format!("{{ {} }}", fields.join(", "));
            output_str.push_str(&format!(
                "    {}: (args: {}): Promise<{}> => window.bridge.send<{}>('{}', args),\n",
                fun_name, arg_type, return_ts, return_ts, fun_name
            ));
        }
    }

    output_str.push_str("};\n\n");

    let output = if fs::exists(&output).is_err() {
        &output.create_parent()?
    } else {
        &output
    };
    fs::write(output, output_str)?;
    Ok(())
}

/// TypeScript 类型表示
#[derive(Debug, Clone, PartialEq)]
enum TsType {
    String,
    Number,
    Boolean,
    Any,
    Null,
    Undefined,
    Array(Box<TsType>),
    Optional(Box<TsType>), // Option<T> 表示为 T | null
}

struct FunInfo {
    name: String,
    param: Vec<(String, TsType)>,
    return_ty: TsType,
}

struct FunCollector<'a> {
    target: Vec<&'a str>,
    funs: Vec<FunInfo>,
    ty_map: HashMap<String, Vec<(String, TsType)>>,
}

impl<'a> FunCollector<'a> {
    fn new(attrs: &[&'a str]) -> Self {
        Self {
            target: attrs.to_vec(),
            funs: Vec::new(),
            ty_map: HashMap::new(),
        }
    }

    fn has_target_attr(&self, attrs: &[Attribute]) -> bool {
        attrs.iter().any(|attr| {
            let path_str = attr
                .path()
                .segments
                .iter()
                .map(|seg| seg.ident.to_string())
                .collect::<Vec<_>>();
            self.target
                .iter()
                .any(|s| path_str.contains(&s.to_string()))
        })
    }
}

impl<'a, 'ast> Visit<'ast> for FunCollector<'a> {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        if self.has_target_attr(&node.attrs)
            && let Ok(info) = parse_fun(node, &mut self.ty_map)
        {
            self.funs.push(info);
        }
        visit_item_fn(self, node);
    }
}

impl TsType {
    fn to_ts_string(&self) -> String {
        match self {
            TsType::String => "string".into(),
            TsType::Number => "number".into(),
            TsType::Boolean => "boolean".into(),
            TsType::Any => "any".into(),
            TsType::Null => "null".into(),
            TsType::Undefined => "undefined".into(),
            TsType::Array(inner) => format!("{}[]", inner.to_ts_string()),
            TsType::Optional(inner) => format!("{} | null", inner.to_ts_string()),
        }
    }
}

fn parse_fun(
    func: &ItemFn,
    type_map: &mut HashMap<String, Vec<(String, TsType)>>,
) -> Result<FunInfo> {
    let name = func.sig.ident.to_string();
    let mut param = Vec::new();

    for input in &func.sig.inputs {
        if let FnArg::Typed(PatType { pat, ty, .. }) = input
            && let Pat::Ident(pat_ident) = &**pat
        {
            let param_name = pat_ident.ident.to_string();
            let ts_ty = type2ts(ty, type_map)?;
            param.push((param_name, ts_ty));
        } else {
            return Err(newerr!("Unsupported"));
        }
    }

    let return_ty = match &func.sig.output {
        syn::ReturnType::Default => TsType::Null,
        syn::ReturnType::Type(_, ty) => return_type2ts(ty, type_map)?,
    };
    Ok(FunInfo {
        name,
        param,
        return_ty,
    })
}

fn return_type2ts(
    ty: &syn::Type,
    type_map: &mut HashMap<String, Vec<(String, TsType)>>,
) -> Result<TsType> {
    if let Type::Path(type_path) = ty {
        let seg = type_path
            .path
            .segments
            .last()
            .ok_or(newerr!("segments.last is null"))?;
        if seg.ident == "Result"
            && let PathArguments::AngleBracketed(args) = &seg.arguments
            && let Some(GenericArgument::Type(inner_ty)) = args.args.first()
        {
            return type2ts(inner_ty, type_map);
        }
    }
    type2ts(ty, type_map)
}

fn type2ts(
    ty: &syn::Type,
    _type_map: &mut HashMap<String, Vec<(String, TsType)>>,
) -> Result<TsType> {
    match ty {
        Type::Path(type_path) => {
            let seg = type_path
                .path
                .segments
                .last()
                .ok_or(newerr!("segments.last is null"))?;
            match seg.ident.to_string().as_str() {
                "String" | "str" => return Ok(TsType::String),
                "bool" => return Ok(TsType::Boolean),
                "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "f32" | "f64"
                | "usize" | "isize" => return Ok(TsType::Number),
                "Vec" => {
                    if let PathArguments::AngleBracketed(args) = &seg.arguments
                        && let Some(GenericArgument::Type(inner_type)) = args.args.first()
                    {
                        return Ok(TsType::Array(Box::new(type2ts(inner_type, _type_map)?)));
                    }
                }
                "()" => return Ok(TsType::Undefined),
                "Option" => {
                    if let PathArguments::AngleBracketed(args) = &seg.arguments
                        && let Some(GenericArgument::Type(inner_type)) = args.args.first()
                    {
                        return Ok(TsType::Optional(Box::new(type2ts(inner_type, _type_map)?)));
                    }
                }
                _name => {
                    // let type_name = name.to_case(Case::Pascal);
                    // todo 解析结构
                    return Ok(TsType::Any);
                }
            }
        }
        Type::Reference(ref_ty) => return type2ts(&ref_ty.elem, _type_map),
        _ => {}
    };
    Ok(TsType::Any)
}
