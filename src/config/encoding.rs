use super::{TypeRef, SignalRef};



// describes how to map Type to signals.
// vector of elements with name and type of the encoded Types
pub type MessageEncoding = Vec<TypeSignalEncoding>;


#[derive(Debug)]
pub struct TypeSignalEncoding {
    name: String,
    ty: TypeRef,
    signals: Vec<SignalRef>,
}

impl TypeSignalEncoding {
    pub fn new(name: String, ty: TypeRef, signals: Vec<SignalRef>) -> TypeSignalEncoding {
        TypeSignalEncoding { name, ty, signals }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn ty(&self) -> &TypeRef {
        &self.ty
    }
    pub fn signals(&self) -> &Vec<SignalRef> {
        &self.signals
    }
}

