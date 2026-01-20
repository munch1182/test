pub fn main() {
    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true)
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .type_attribute(".", "#[serde(rename_all = \"camelCase\")]")
        .compile_protos(&["proto/plugin.proto"], &["proto"])
        .unwrap();
}
