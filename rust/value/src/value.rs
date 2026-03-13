use crate::Number;
use std::{collections::HashMap, sync::Arc};

/// 动态值类型，可表示 rust 的各种数据类型。
///
/// 该枚举支持空值、布尔、数字、字符串、二进制数据、数组和映射。
/// 字符串和二进制数据使用 `Arc` 实现共享所有权，克隆廉价。
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// 空值，对应 JSON 的 null。
    Null,
    /// 布尔值。
    Bool(bool),
    /// 数字，使用 [`Number`] 枚举统一表示多种数值类型。
    Number(Number),
    /// UTF-8 字符串，内部使用 `Arc<str>` 实现共享。
    String(Arc<str>),
    /// 二进制数据，内部使用 `Arc<[u8]>` 实现共享。
    Bytes(Arc<[u8]>),
    /// 值数组。
    Array(Vec<Value>),
    /// 键值对映射，键为字符串。
    Map(HashMap<String, Value>),
}

macro_rules! impl_as_type {
    ($( ($fn_name:ident, $enum:ident, $ty:ty) ),* $(,)?) => {
        impl Value {
            $(
                pub fn $fn_name(&self) -> Option<&$ty> {
                    match self {
                        Value::$enum(val) => impl_as_type!(@convert $enum, val), // 需要根据 $enum 选择正确的转换方式
                        _ => None,
                    }
                }
            )*
        }
    };
    // 针对不同变体的转换规则
    (@convert String, $val:ident) => { Some($val.as_ref()) };   // Arc<str> → &str
    (@convert Bytes, $val:ident)  => { Some($val.as_ref()) };   // Arc<[u8]> → &[u8]
    (@convert Bool, $val:ident)   => { Some($val) };            // bool → &bool
    (@convert Number, $val:ident) => { Some($val) };            // Number → &Number
    (@convert Array, $val:ident)  => { Some($val) };            // Vec<Value> → &Vec<Value>
    (@convert Map, $val:ident)    => { Some($val) };            // HashMap → &HashMap
}

impl_as_type!(
    (as_str, String, str),
    (as_bool, Bool, bool),
    (as_bytes, Bytes, [u8]),
    (as_number, Number, Number),
    (as_array, Array, Vec<Value>),
    (as_map, Map, HashMap<String, Value>),
);

macro_rules! impl_from_value {
    ($( $unit:tt ),* $(,)?) => {
        $(
            impl_from_value!(@impl $unit);
        )*
    };

    (@impl ( $enum:ident => $ty:ty )) => {
        impl From<$ty> for Value {
            fn from(val: $ty) -> Self {
                Value::$enum(val)
            }
        }
        impl TryFrom<Value> for $ty {
            type Error = ValueParseError;
            fn try_from(value: Value) -> Result<Self, Self::Error> {
                match value {
                    Value::$enum(val) => Ok(val),
                    _ => Err(ValueParseError),
                }
            }
        }
    };

    (@impl ( ===> $ty:ty )) => {
        impl From<$ty> for Value {
            fn from(value: $ty) -> Self {
                Value::Number(Number::from(value))
            }
        }
        impl TryFrom<Value> for $ty {
            type Error = ValueParseError;
            fn try_from(value: Value) -> Result<Self, Self::Error> {
                Number::try_from(value)?.try_into()
            }
        }
    };

    (@impl ($enum:ident => $ty:ty, ($concert_to:expr; $concert_from:expr) )) => {
        impl From<$ty> for Value {
            fn from(val: $ty) -> Self {
                Value::$enum($concert_to(val))
            }
        }
        impl TryFrom<Value> for $ty {
            type Error = ValueParseError;
            fn try_from(value: Value) -> Result<Self, Self::Error> {
                match value {
                    Value::$enum(val) => Ok($concert_from(val)),
                    _ => Err(ValueParseError),
                }
            }
        }
    };
}

impl_from_value!(
    (Bool => bool),
    (Number => Number),
    (String => String, (|s| Arc::from(s); |s: Arc<str>| s.to_string())),
    (String => Arc<str>),
    (Bytes => Arc<[u8]>),
    ( ===> i8),
    ( ===> i16),
    ( ===> i32),
    ( ===> i64),
    ( ===> u8),
    ( ===> u16),
    ( ===> u32),
    ( ===> u64),
    ( ===> f32),
    ( ===> f64)
);

impl<T: Into<Value>> From<Vec<T>> for Value {
    fn from(vec: Vec<T>) -> Self {
        Value::Array(vec.into_iter().map(Into::into).collect())
    }
}

// 为 Vec<T> 实现 TryFrom
impl<T: TryFrom<Value, Error = ValueParseError>> TryFrom<Value> for Vec<T> {
    type Error = ValueParseError;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Array(vec) => vec.into_iter().map(T::try_from).collect(),
            _ => Err(ValueParseError),
        }
    }
}

impl<T: Into<Value>> From<HashMap<String, T>> for Value {
    fn from(map: HashMap<String, T>) -> Self {
        Value::Map(map.into_iter().map(|(k, v)| (k, v.into())).collect())
    }
}

impl<T: TryFrom<Value, Error = ValueParseError>> TryFrom<Value> for HashMap<String, T> {
    type Error = ValueParseError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Map(map) => map
                .into_iter()
                .map(|(k, v)| Ok((k, T::try_from(v)?)))
                .collect(),
            _ => Err(ValueParseError),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValueParseError;

impl std::error::Error for ValueParseError {}

impl std::fmt::Display for ValueParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "value parse error")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from() {
        let a = 31;
        let value = Value::from(Number::from(a));

        let f = value.as_bool();
        assert!(f.is_none());

        let num = value.as_number();
        assert!(num.is_some());

        assert_eq!(num.unwrap(), &Number::from(a));

        let number = Number::try_from(value);
        assert!(number.is_ok());
    }

    #[test]
    fn test_vec_conversion() {
        let original = vec![1, 2, 3];
        let value = Value::from(original.clone());
        assert!(matches!(value, Value::Array(_)));

        let recovered: Result<Vec<i32>, ValueParseError> = Vec::try_from(value);
        assert!(recovered.is_ok());
        assert_eq!(recovered.unwrap(), original);
    }
}
