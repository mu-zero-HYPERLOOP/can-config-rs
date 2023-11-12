use super::{TypeRef, SignalRef};



// describes how to map Type to signals.
// vector of elements with name and type of the encoded Types

#[derive(Debug)]
pub struct MessageEncoding {
    attributes : Vec<TypeSignalEncoding>,
}

impl MessageEncoding {
    pub fn new(attributes : Vec<TypeSignalEncoding>) -> Self{
        Self {
            attributes
        }
    }
    pub fn attributes(&self) -> &Vec<TypeSignalEncoding> {
        &self.attributes
    }
}

#[derive(Debug)]
pub enum TypeSignalEncoding {
    Composite(CompositeSignalEncoding),
    Primitive(PrimitiveSignalEncoding),
}

impl TypeSignalEncoding {
    pub fn name(&self) -> &str {
        match &self {
            TypeSignalEncoding::Composite(comp) => comp.name(),
            TypeSignalEncoding::Primitive(prim) => prim.name(),
        }
    }
    pub fn ty(&self) -> &TypeRef {
        match &self {
            TypeSignalEncoding::Composite(comp) => comp.ty(),
            TypeSignalEncoding::Primitive(prim) => prim.ty(),
        }
    }
}

#[derive(Debug)]
pub struct CompositeSignalEncoding {
    composite_name : String,
    attributes : Vec<TypeSignalEncoding>,
    ty : TypeRef,
}

impl CompositeSignalEncoding {
    pub fn new(composite_name : String,
            attributes : Vec<TypeSignalEncoding>, ty : TypeRef) -> Self {
        Self {
            composite_name,
            attributes,
            ty,
        }
    }
    pub fn name(&self) -> &str {
        &self.composite_name
    }
    pub fn attributes(&self) -> &Vec<TypeSignalEncoding> {
        &self.attributes
    }
    pub fn ty(&self) -> &TypeRef {
        &self.ty
    }
}

#[derive(Debug)]
pub struct PrimitiveSignalEncoding {
    name : String,
    ty : TypeRef,
    signal : SignalRef,
}

impl PrimitiveSignalEncoding {
    pub fn new(name : String,
               ty : TypeRef,
               signal : SignalRef) -> Self {
        Self {
            name,
            ty,
            signal
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn ty(&self) -> &TypeRef {
        &self.ty
    }
    pub fn signal(&self) -> &SignalRef {
        &self.signal
    }
}

// #[derive(Debug)]
// pub struct TypeSignalEncoding {
//     name: String,
//     ty: TypeRef,
//     signals: Vec<SignalRef>,
// }
//
// impl TypeSignalEncoding {
//     pub fn new(name: String, ty: TypeRef, signals: Vec<SignalRef>) -> TypeSignalEncoding {
//         TypeSignalEncoding { name, ty, signals }
//     }
//     pub fn name(&self) -> &str {
//         &self.name
//     }
//     pub fn ty(&self) -> &TypeRef {
//         &self.ty
//     }
//     pub fn signals(&self) -> &Vec<SignalRef> {
//         &self.signals
//     }
// }
//
