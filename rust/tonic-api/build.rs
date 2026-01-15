// build.rs - 生成Rust代码
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {

    // 配置Tonic构建
    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true)
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .type_attribute(".", "#[serde(rename_all = \"camelCase\")]")
        .field_attribute("created_at", "#[serde(with = \"crate::utils::timestamp\")]")
        .compile_protos(&["proto/echo.proto"], &["proto"])?;

    // 重新运行如果proto文件变化
    println!("cargo:rerun-if-changed=proto/");

    Ok(())
}
