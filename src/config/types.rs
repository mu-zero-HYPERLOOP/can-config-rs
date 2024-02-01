use super::{ConfigRef, SignalType, Visibility};

pub type TypeRef = ConfigRef<Type>;

#[derive(Debug, PartialEq)]
pub enum Type {
    Primitive(SignalType),
    Struct {
        name: String,
        description: Option<String>,
        attribs: Vec<(String, TypeRef)>,
        visibility: Visibility,
    },
    Enum {
        name: String,
        description: Option<String>,
        size: u8,
        entries: Vec<(String, u64)>,
        visibility: Visibility,
    },
    Array {
        len: usize,
        ty: TypeRef,
    },
}

impl Type {
    pub fn name(&self) -> String {
        match &self {
            Type::Primitive(signal_type) => match signal_type {
                SignalType::UnsignedInt { size } => {
                    return format!("u{size}");
                }
                SignalType::SignedInt { size } => {
                    return format!("i{size}");
                }
                SignalType::Decimal {
                    size,
                    offset,
                    scale,
                } => {
                    return format!("d{size}<offset={offset}, scale={scale}>");
                }
            },
            Type::Struct {
                name,
                description: _,
                attribs: _,
                visibility: _,
            } => name.to_owned(),
            Type::Enum {
                name,
                description: _,
                size: _,
                entries: _,
                visibility: _,
            } => name.to_owned(),
            Type::Array { len, ty } => format!("{}[{len}]", ty.name()),
        }
    }
    pub fn size(&self) -> u32 {
        match &self {
            Type::Primitive(signal_type) => signal_type.size() as u32,
            Type::Struct {
                name: _,
                description: _,
                attribs,
                visibility: _,
            } => attribs.iter().map(|(_, attrib_ty)| attrib_ty.size()).sum(),
            Type::Enum {
                name: _,
                description: _,
                size,
                entries: _,
                visibility: _,
            } => *size as u32,
            Type::Array { len, ty } => ty.size() * *len as u32,
        }
    }
}
