use core::fmt::*;
use utf16string::LittleEndian;
use winapi::shared::{devpropdef::DEVPROPTYPE, guiddef::GUID};

pub enum DevProperty {
    Empty,
    Null,
    I8(i8),
    U8(u8),
    I16(i16),
    U16(u16),
    I32(i32),
    U32(u32),
    I64(i64),
    U64(u64),
    F32(f32),
    F64(f64),
    Bool(bool),
    Guid(GUID),
    Binary(Box<[u8]>),
    String(utf16string::WString<LittleEndian>),
    I8Array(Box<[i8]>),
    U8Array(Box<[u8]>),
    I16Array(Box<[i16]>),
    U16Array(Box<[u16]>),
    I32Array(Box<[i32]>),
    U32Array(Box<[u32]>),
    I64Array(Box<[i64]>),
    U64Array(Box<[u64]>),
    F32Array(Box<[f32]>),
    F64Array(Box<[f64]>),
    BoolArray(Box<[bool]>),
    GuidArray(Box<[GUID]>),
    Unsupported(DEVPROPTYPE),
}

impl Debug for DevProperty {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        use DevProperty::*;

        macro tuple($fmt:ident, $name:literal, $value:expr) {
            $fmt.debug_tuple(concat!("DevProperty::", $name))
                .field($value)
                .finish()
        }

        match self {
            Empty => write!(f, "DevProperty::Empty"),
            Null => write!(f, "DevProperty::Null"),
            Bool(v) => tuple!(f, "Bool", v),
            String(v) => tuple!(f, "String", v),
            I8(v) => tuple!(f, "I8", v),
            U8(v) => tuple!(f, "U8", v),
            I16(v) => tuple!(f, "I16", v),
            U16(v) => tuple!(f, "U16", v),
            I32(v) => tuple!(f, "I32", v),
            U32(v) => tuple!(f, "U32", v),
            I64(v) => tuple!(f, "I64", v),
            U64(v) => tuple!(f, "U64", v),
            F32(v) => tuple!(f, "F32", v),
            F64(v) => tuple!(f, "F64", v),
            Guid(v) => tuple!(f, "Guid", &fmt::Guid(v)),
            BoolArray(v) => tuple!(f, "BoolArray", v),
            I8Array(v) => tuple!(f, "I8Array", v),
            U8Array(v) => tuple!(f, "U8Array", v),
            I16Array(v) => tuple!(f, "I16Array", v),
            U16Array(v) => tuple!(f, "U16Array", v),
            I32Array(v) => tuple!(f, "I32Array", v),
            U32Array(v) => tuple!(f, "U32Array", v),
            I64Array(v) => tuple!(f, "I64Array", v),
            U64Array(v) => tuple!(f, "U64Array", v),
            F32Array(v) => tuple!(f, "F32Array", v),
            F64Array(v) => tuple!(f, "F64Array", v),
            GuidArray(v) => tuple!(f, "GuidArray", &fmt::GuidSlice(v)),
            Binary(v) => tuple!(f, "Binary", v),
            Unsupported(v) => tuple!(f, "Unsupported", v),
        }
    }
}

impl Display for DevProperty {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        use DevProperty::*;
        match self {
            Empty => write!(f, "#EMPTY"),
            Null => write!(f, "#NULL"),
            Bool(v) => write!(f, "{v}"),
            String(v) => write!(f, "{}", v.to_utf8()),
            I8(v) => write!(f, "{v}"),
            U8(v) => write!(f, "{v}"),
            I16(v) => write!(f, "{v}"),
            U16(v) => write!(f, "{v}"),
            I32(v) => write!(f, "{v}"),
            U32(v) => write!(f, "{v}"),
            I64(v) => write!(f, "{v}"),
            U64(v) => write!(f, "{v}"),
            F32(v) => write!(f, "{v}"),
            F64(v) => write!(f, "{v}"),
            Guid(v) => write!(f, "{}", fmt::Guid(v)),
            BoolArray(v) => write!(f, "{v:?}"),
            I8Array(v) => write!(f, "{v:?}"),
            U8Array(v) => write!(f, "{v:?}"),
            I16Array(v) => write!(f, "{v:?}"),
            U16Array(v) => write!(f, "{v:?}"),
            I32Array(v) => write!(f, "{v:?}"),
            U32Array(v) => write!(f, "{v:?}"),
            I64Array(v) => write!(f, "{v:?}"),
            U64Array(v) => write!(f, "{v:?}"),
            F32Array(v) => write!(f, "{v:?}"),
            F64Array(v) => write!(f, "{v:?}"),
            GuidArray(v) => write!(f, "{}", fmt::GuidSlice(v)),
            Binary(v) => v.iter().try_for_each(|v| write!(f, "{v:02x}")),
            Unsupported(v) => write!(f, "#UNSUP{{{v}}}"),
        }
    }
}

pub mod fmt {
    use super::*;

    /// Utility struct for formatting a [`GUID`]
    pub struct Guid<'a>(pub &'a GUID);

    impl Debug for Guid<'_> {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            f.debug_struct("Guid")
                .field("Data1", &self.0.Data1)
                .field("Data2", &self.0.Data2)
                .field("Data3", &self.0.Data3)
                .field("Data4", &self.0.Data4)
                .finish()
        }
    }

    impl Display for Guid<'_> {
        fn fmt(&self, fmt: &mut Formatter<'_>) -> Result {
            let GUID {
                Data1: a,
                Data2: b,
                Data3: c,
                Data4: [d, e, f, g, h, i, j, k],
            } = self.0;
            write!(
                fmt,
                "{a:08x}-{b:04x}-{c:04x}-{d:02x}{e:02x}-{f:02x}{g:02x}{h:02x}{i:02x}{j:02x}{k:02x}"
            )
        }
    }

    /// Utility struct for formatting a [slice](std::slice) of [`GUID`]s
    pub struct GuidSlice<'a>(pub &'a [GUID]);

    impl Debug for GuidSlice<'_> {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            f.debug_list().entries(self.0.iter().map(Guid)).finish()
        }
    }

    impl Display for GuidSlice<'_> {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            let Some((first, rest)) = self.0.split_first() else { return write!(f, "[]") };

            let start = if f.alternate() { "[\n    " } else { "[" };
            write!(f, "{start}{}", Guid(first))?;

            let between = if f.alternate() { ",\n    " } else { ", " };
            for guid in rest {
                write!(f, "{between}{}", Guid(guid))?;
            }
            let end = if f.alternate() { "\n]" } else { "]" };
            write!(f, "{end}")
        }
    }
}
