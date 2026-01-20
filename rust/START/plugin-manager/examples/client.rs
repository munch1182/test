use plugin_manager::{PluginReq, PluginServerClient};

#[tokio::main]
pub async fn main() {
    let addr = "http://127.0.0.1:50051";

    let mut client = PluginServerClient::connect(addr).await.unwrap();

    let req = PluginReq {
        id: "43f551f96ddd5314b63cb342".to_string(),
        data: "list".as_bytes().to_vec(),
    };
    let res = client.execute(req).await.unwrap();
    let data = res.into_inner().data;
    let data = String::from_utf8(data).unwrap();
    println!("data: {}", data)
}
