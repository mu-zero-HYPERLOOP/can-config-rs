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
}
