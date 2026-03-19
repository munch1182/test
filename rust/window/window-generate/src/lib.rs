use libcommon::{ext::FileDirCreateExt, newerr, prelude::Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use syn::{
    Attribute, FnArg, GenericArgument, ItemEnum, ItemFn, ItemStruct, Pat, PatType, PathArguments,
    Type,
    visit::{Visit, visit_item_enum, visit_item_fn, visit_item_struct},
};

const NO_INPUT_PARAM_ATTR: [&str; 1] = ["WindowState"];

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
    let mut output_str = String::new();

    output_str.push_str("// This file is auto-generated. Do not edit manually.\n\n");

    // 输出所有结构体接口
    for (ty_name, fields) in &collector.ty_map {
        output_str.push_str(&format!("export interface {ty_name} {{\n"));
        for (field_name, field_ty) in fields {
            output_str.push_str(&format!("    {field_name}: {};\n", field_ty.to_ts_string()));
        }
        output_str.push_str("}\n\n");
    }

    // 输出所有枚举类型
    for (enum_name, variants) in &collector.enum_ts_map {
        match variants {
            EnumTsType::Unit(variants) => {
                let variants_str = variants
                    .iter()
                    .map(|v| format!("\"{}\"", v))
                    .collect::<Vec<_>>()
                    .join(" | ");
                output_str.push_str(&format!("export type {enum_name} = {variants_str};\n\n"));
            }
            EnumTsType::Complex(variants) => {
                let mut union_parts = Vec::new();
                for (variant_name, fields) in variants {
                    let mut field_lines = vec![format!("    type: \"{}\";", variant_name)];
                    for (field_name, field_ty) in fields {
                        field_lines.push(format!(
                            "    {}: {};",
                            field_name,
                            field_ty.to_ts_string()
                        ));
                    }
                    let obj = format!("{{\n{}\n}}", field_lines.join("\n"));
                    union_parts.push(obj);
                }
                output_str.push_str(&format!(
                    "export type {enum_name} = {};\n\n",
                    union_parts.join(" | ")
                ));
            }
        }
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
    Object(String),        // 引用自定义类型（结构体或枚举）
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
            TsType::Object(name) => name.clone(),
        }
    }
}

/// 用于存储枚举的 TypeScript 表示
#[derive(Debug, Clone)]
enum EnumTsType {
    /// 单元枚举：所有变体都是无字段的，生成联合类型
    Unit(Vec<String>),
    /// 复杂枚举：变体可能有字段，生成带 type 标签的对象联合
    Complex(Vec<(String, Vec<(String, TsType)>)>),
}

struct FunInfo {
    name: String,
    param: Vec<(String, TsType)>,
    return_ty: TsType,
}

struct FunCollector<'a> {
    target: Vec<&'a str>,
    funs: Vec<FunInfo>,
    // 存储结构体定义（AST），用于后续解析字段
    structs: HashMap<String, ItemStruct>,
    // 存储枚举定义（AST）
    enums: HashMap<String, ItemEnum>,
    // 存储已经解析完成的结构体字段映射（类型名 -> 字段列表）
    ty_map: HashMap<String, Vec<(String, TsType)>>,
    // 存储已经解析完成的枚举映射
    enum_ts_map: HashMap<String, EnumTsType>,
}

impl<'a> FunCollector<'a> {
    fn new(attrs: &[&'a str]) -> Self {
        Self {
            target: attrs.to_vec(),
            funs: Vec::new(),
            structs: HashMap::new(),
            enums: HashMap::new(),
            ty_map: HashMap::new(),
            enum_ts_map: HashMap::new(),
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

    /// 解析一个类型，并递归填充 ty_map 和 enum_ts_map
    fn resolve_type(&mut self, ty: &Type) -> Result<TsType> {
        match ty {
            Type::Path(type_path) => {
                let seg = type_path
                    .path
                    .segments
                    .last()
                    .ok_or(newerr!("segments.last is null"))?;
                let ident = seg.ident.to_string();

                // 基本类型
                match ident.as_str() {
                    "String" | "str" => Ok(TsType::String),
                    "bool" => Ok(TsType::Boolean),
                    "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "f32" | "f64"
                    | "usize" | "isize" => Ok(TsType::Number),
                    "Vec" => {
                        if let PathArguments::AngleBracketed(args) = &seg.arguments
                            && let Some(GenericArgument::Type(inner_type)) = args.args.first()
                        {
                            let inner = self.resolve_type(inner_type)?;
                            return Ok(TsType::Array(Box::new(inner)));
                        }
                        Ok(TsType::Any)
                    }
                    "()" => Ok(TsType::Undefined),
                    "Option" => {
                        if let PathArguments::AngleBracketed(args) = &seg.arguments
                            && let Some(GenericArgument::Type(inner_type)) = args.args.first()
                        {
                            let inner = self.resolve_type(inner_type)?;
                            return Ok(TsType::Optional(Box::new(inner)));
                        }
                        Ok(TsType::Any)
                    }
                    _ => {
                        // 自定义类型：可能是结构体或枚举
                        // 先检查是否已经解析过
                        if self.ty_map.contains_key(&ident) || self.enum_ts_map.contains_key(&ident)
                        {
                            return Ok(TsType::Object(ident));
                        }

                        // 尝试从 structs 中移除并解析（避免同时持有不可变引用和可变借用）
                        if let Some(item_struct) = self.structs.remove(&ident) {
                            self.parse_struct(&item_struct)?;
                            return Ok(TsType::Object(ident));
                        }

                        // 尝试从 enums 中移除并解析
                        if let Some(item_enum) = self.enums.remove(&ident) {
                            self.parse_enum(&item_enum)?;
                            return Ok(TsType::Object(ident));
                        }

                        // 未找到，视为 any
                        Ok(TsType::Any)
                    }
                }
            }
            Type::Reference(ref_ty) => self.resolve_type(&ref_ty.elem),
            Type::Slice(slice_ty) => {
                let inner = self.resolve_type(&slice_ty.elem)?;
                Ok(TsType::Array(Box::new(inner)))
            }
            Type::Array(array_ty) => {
                let inner = self.resolve_type(&array_ty.elem)?;
                Ok(TsType::Array(Box::new(inner)))
            }
            _ => Ok(TsType::Any),
        }
    }

    /// 解析结构体，填充 ty_map（处理自引用：先插入空字段再解析）
    fn parse_struct(&mut self, item: &ItemStruct) -> Result<()> {
        let name = item.ident.to_string();
        if self.ty_map.contains_key(&name) {
            return Ok(());
        }

        // 预先插入空字段，防止递归解析自身时无限循环
        self.ty_map.insert(name.clone(), Vec::new());

        let mut fields = Vec::new();
        match &item.fields {
            syn::Fields::Named(fields_named) => {
                for field in &fields_named.named {
                    if let Some(ident) = &field.ident {
                        let field_name = ident.to_string();
                        let field_ty = self.resolve_type(&field.ty)?;
                        fields.push((field_name, field_ty));
                    }
                }
            }
            syn::Fields::Unnamed(fields_unnamed) => {
                for (i, field) in fields_unnamed.unnamed.iter().enumerate() {
                    let field_name = format!("field{}", i); // 元组结构体字段命名为 field0, field1...
                    let field_ty = self.resolve_type(&field.ty)?;
                    fields.push((field_name, field_ty));
                }
            }
            syn::Fields::Unit => {} // 单元结构体没有字段
        }

        // 更新为真正的字段列表
        self.ty_map.insert(name, fields);
        Ok(())
    }

    /// 解析枚举，填充 enum_ts_map
    fn parse_enum(&mut self, item: &ItemEnum) -> Result<()> {
        let name = item.ident.to_string();
        if self.enum_ts_map.contains_key(&name) {
            return Ok(());
        }

        // 判断是否所有变体都是单元变体
        let all_unit = item
            .variants
            .iter()
            .all(|v| matches!(v.fields, syn::Fields::Unit));

        if all_unit {
            let variant_names: Vec<String> =
                item.variants.iter().map(|v| v.ident.to_string()).collect();
            self.enum_ts_map
                .insert(name, EnumTsType::Unit(variant_names));
            Ok(())
        } else {
            let mut variants_info = Vec::new();
            for variant in &item.variants {
                let variant_name = variant.ident.to_string();
                let mut fields = Vec::new();
                match &variant.fields {
                    syn::Fields::Named(fields_named) => {
                        for field in &fields_named.named {
                            if let Some(ident) = &field.ident {
                                let field_name = ident.to_string();
                                let field_ty = self.resolve_type(&field.ty)?;
                                fields.push((field_name, field_ty));
                            }
                        }
                    }
                    syn::Fields::Unnamed(fields_unnamed) => {
                        for (i, field) in fields_unnamed.unnamed.iter().enumerate() {
                            let field_name = format!("field{}", i);
                            let field_ty = self.resolve_type(&field.ty)?;
                            fields.push((field_name, field_ty));
                        }
                    }
                    syn::Fields::Unit => {} // 无字段
                }
                variants_info.push((variant_name, fields));
            }
            self.enum_ts_map
                .insert(name, EnumTsType::Complex(variants_info));
            Ok(())
        }
    }
}

impl<'a, 'ast> Visit<'ast> for FunCollector<'a> {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        if self.has_target_attr(&node.attrs)
            && let Ok(info) = parse_fun(node, self)
        {
            self.funs.push(info);
        }
        visit_item_fn(self, node);
    }

    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        let name = node.ident.to_string();
        self.structs.insert(name, node.clone());
        visit_item_struct(self, node);
    }

    fn visit_item_enum(&mut self, node: &'ast ItemEnum) {
        let name = node.ident.to_string();
        self.enums.insert(name, node.clone());
        visit_item_enum(self, node);
    }
}

// 解析函数，使用 collector 来解析类型
fn parse_fun(func: &ItemFn, collector: &mut FunCollector) -> Result<FunInfo> {
    let name = func.sig.ident.to_string();
    let mut param = Vec::new();

    for input in &func.sig.inputs {
        if let FnArg::Typed(PatType { pat, ty, .. }) = input {
            let is_no_param = if let Type::Path(ty_path) = &**ty {
                ty_path
                    .path
                    .segments
                    .last()
                    .is_some_and(|s| NO_INPUT_PARAM_ATTR.iter().any(|v| s.ident == v))
            } else {
                false
            };
            if !is_no_param && let Pat::Ident(pat_ident) = &**pat {
                let param_name = pat_ident.ident.to_string();
                let ts_ty = collector.resolve_type(ty)?;
                param.push((param_name, ts_ty));
            }
        } else {
            return Err(newerr!("Unsupported parameter pattern"));
        }
    }

    let return_ty = match &func.sig.output {
        syn::ReturnType::Default => TsType::Null,
        syn::ReturnType::Type(_, ty) => return_type2ts(ty, collector)?,
    };
    Ok(FunInfo {
        name,
        param,
        return_ty,
    })
}

fn return_type2ts(ty: &syn::Type, collector: &mut FunCollector) -> Result<TsType> {
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
            return collector.resolve_type(inner_ty);
        }
    }
    collector.resolve_type(ty)
}
