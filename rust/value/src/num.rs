#[derive(Clone, PartialEq)]
pub enum Number {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
}

impl std::fmt::Debug for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::fmt::Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Number::I8(n) => write!(f, "{}i8", n),
            Number::I16(n) => write!(f, "{}i16", n),
            Number::I32(n) => write!(f, "{}i32", n),
            Number::I64(n) => write!(f, "{}i64", n),
            Number::U8(n) => write!(f, "{}u8", n),
            Number::U16(n) => write!(f, "{}u16", n),
            Number::U32(n) => write!(f, "{}u32", n),
            Number::U64(n) => write!(f, "{}u64", n),
            Number::F32(n) => write!(f, "{}f32", n),
            Number::F64(n) => write!(f, "{}f64", n),
        }
    }
}

macro_rules! number_conversions {
    // 匹配零个或多个条目，每个条目的格式为：variant => type, method_name
    // 可选的尾随逗号用 $(,)? 处理
    ($( $variant:ident => $ty:ty, $fun:ident ),* $(,)?) => {
        $(
            // 从原始类型到 Number 的转换
            impl From<$ty> for Number { fn from(val: $ty) -> Self { Number::$variant(val) } }

            // 从 Number 到原始类型的 TryFrom 转换
            impl TryFrom<Number> for $ty {
                type Error = crate::ValueParseError;
                fn try_from(value: Number) -> Result<Self, Self::Error> {
                    match value {
                        Number::$variant(num) => Ok(num),
                        _ => Err(crate::ValueParseError),
                    }
                }
            }

            // 为 Number 添加访问方法
            impl Number {
                pub fn $fun(&self) -> Option<$ty> {
                    match self {
                        Number::$variant(num) => Some(*num),
                        _ => None,
                    }
                }
            }
        )*
    };
}

number_conversions!(
    I8 => i8, as_i8 ,
    I16 => i16, as_i16,
    I32 => i32, as_i32,
    I64 => i64, as_i64,
    U8=> u8, as_u8,
    U16=> u16, as_u16,
    U32=> u32, as_u32,
    U64=> u64, as_u64,
    F32=> f32, as_f32,
    F64=> f64, as_f64,
);
