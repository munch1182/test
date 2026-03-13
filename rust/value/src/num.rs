
#[derive(Debug, Clone, PartialEq)]
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

macro_rules! impl_from_number {
    () => {};
    ($variant:ident => $ty:ty) => {
        impl From<$ty> for Number { fn from(val: $ty) -> Self { Number::$variant(val)}}
    };
    ($variant:ident => $ty:ty, $($rest:tt)*) => {
        impl_from_number!($variant => $ty);
        impl_from_number!($($rest)*);
    };
}

macro_rules! impl_try_from_number {
    () => {};
    ($variant:ident => $ty:ty) => {
        impl TryFrom<Number> for $ty {
            type Error = crate::ValueParseError;
            fn try_from(value: Number) -> Result<Self, Self::Error> {
                match value {
                    Number::$variant(num) => Ok(num),
                    _ => Err(crate::ValueParseError),
                }
            }
        }
    };

    ($variant:ident => $ty:ty, $($rest:tt)*) => {
        impl_try_from_number!($variant => $ty);
        impl_try_from_number!($($rest)*);
    };
}

impl_from_number!(
    I8 => i8,
    I16 => i16,
    I32 => i32,
    I64 => i64,
    U8=> u8,
    U16=> u16,
    U32=> u32,
    U64=> u64,
    F32=> f32,
    F64=> f64
);

impl_try_from_number!(
    I8 => i8,
    I16 => i16,
    I32 => i32,
    I64 => i64,
    U8=> u8,
    U16=> u16,
    U32=> u32,
    U64=> u64,
    F32=> f32,
    F64=> f64
);
