use crate::prelude::*;

#[async_trait]
pub trait Plugin: Send + Sync {
    async fn call(&self, input: Value) -> Result<Value, Box<dyn std::error::Error + Send + Sync>>;
}
