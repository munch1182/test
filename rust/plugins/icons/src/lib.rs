use plugin::{Plugin, PluginResult, Value};
use plugin_macro::Value;

#[unsafe(no_mangle)]
pub fn plugin() -> Box<dyn Plugin> {
    Box::new(IconPlugin)
}

#[derive(Debug, Default)]
struct IconPlugin;

#[async_trait::async_trait]
impl Plugin for IconPlugin {
    async fn call(&self, input: Value) -> PluginResult<Value> {
        let a = Input::try_from(input)?;
        match Call::try_from(a.name)? {
            Call::Create => Ok(Value::Number(plugin::Number::U8(a.param + 1u8))),
        }
    }
}

#[derive(Debug, Value)]
struct Input {
    name: u8,
    param: u8,
}

enum Call {
    Create,
}

impl TryFrom<u8> for Call {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Call::Create),
            _ => Err("Unknown call"),
        }
    }
}
