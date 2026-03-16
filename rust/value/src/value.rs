use crate::Number;
use std::{collections::HashMap, convert::Infallible, sync::Arc};

/// 动态值类型，可表示 rust 的各种数据类型。
///
/// 该枚举支持空值、布尔、数字、字符串、二进制数据、数组和映射。
/// 字符串和二进制数据使用 `Arc` 实现共享所有权，克隆廉价。
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// 空值
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

#[macro_export]
macro_rules! value_conversions {
     ($( $unit:tt ),* $(,)?)=>{
        $(
            value_conversions!(@impl $unit);
        )*
    };
    (@impl ($enum:ident => $ty:ty)) => {
        value_conversions!(@impl ($enum => $ty: (|v|v; |v|v)));
    };
    (@impl ($enum:ident ==> $ty:ty)) => {
        value_conversions!(@impl ($enum ==> $ty: (|v|v; |v|v)));
    };
    (@impl ($enum:ident => $ty:ty: ($from:expr; $to:expr)))=> {
        impl From<$ty> for Value {
            fn from(value: $ty) -> Self {
                Value::$enum($from(value))
            }
        }
        impl TryFrom<Value> for $ty {
            type Error = ValueParseError;
            fn try_from(value: Value) -> Result<Self, Self::Error> {
                match value {
                    Value::$enum(val) => Ok($to(val)),
                    _ => Err(ValueParseError),
                }
            }
        }
    };
    (@impl ($enum:ident ==> $ty:ty: ($from:expr; $to:expr)))=> {
        impl From<$ty> for Value {
            fn from(value: $ty) -> Self {
                Value::Number($crate::Number::$enum($from(value)))
            }
        }
        impl TryFrom<Value> for $ty {
            type Error = ValueParseError;
            fn try_from(value: Value) -> Result<Self, Self::Error> {
                match value {
                    Value::Number($crate::Number::$enum(val)) => Ok($to(val)),
                    _ => Err(ValueParseError),
                }
            }
        }
    };
}

value_conversions!(
    (Bool => bool),
    (String => String: (Arc::from; |v:Arc<str>|v.to_string())),
    (Bytes => Arc<[u8]>),
    (Number => Number),
    (I8 ==> i8),
    (U8 ==> u8),
    (I16 ==> i16),
    (U16 ==> u16),
    (I32 ==> i32),
    (U32 ==> u32),
    (I64 ==> i64),
    (U64 ==> u64),
    (F32 ==> f32),
    (F64 ==> f64),
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

impl<T: Into<Value>> From<Option<T>> for Value {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(v) => v.into(),
            None => Value::Null,
        }
    }
}

impl<T: TryFrom<Value, Error = ValueParseError>> TryFrom<Value> for Option<T> {
    type Error = ValueParseError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Null => Ok(None),
            v => Ok(Some(T::try_from(v)?)),
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

///
/// 标准库实现的impl<T, U> TryFrom<U> for T where U: Into<T>返回错误类型Infallible;
/// 会导致Value类型本身转为Value类型时，返回错误类型冲突;
/// 因此需要添加此自动转换;
/// 
impl From<Infallible> for ValueParseError {
    fn from(_value: Infallible) -> Self {
        Self
    }
}