use value::Value;

pub type PluginResult<T> = Result<T, Box<dyn std::error::Error>>;

#[async_trait::async_trait]
pub trait Plugin: Send + Sync {
    async fn call(&self, input: &Value) -> PluginResult<Value>;
}
