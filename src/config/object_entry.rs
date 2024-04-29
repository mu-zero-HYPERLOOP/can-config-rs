use std::{hash::Hash, sync::OnceLock};

use super::{ConfigRef, TypeRef, Visibility, NodeRef};


pub type ObjectEntryRef = ConfigRef<ObjectEntry>;

#[derive(Debug, Clone, Hash)]
pub enum ObjectEntryAccess {
    Const,  // no write
    Local,  // local write public read
    Global, // public write
}

#[derive(Debug)]
pub struct ObjectEntry {
    name: String,
    description: Option<String>,
    unit : Option<String>,
    id: u32,
    ty: TypeRef,
    access: ObjectEntryAccess,
    visibility: Visibility,
    node : OnceLock<NodeRef>,
}

impl Hash for ObjectEntry {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.unit.hash(state);
        self.id.hash(state);
        self.ty.hash(state);
        self.access.hash(state);
        self.visibility.hash(state);
    }
}

impl ObjectEntry {
    pub fn new(name : String, description : Option<String>,
               unit : Option<String>,
               id : u32,
               ty : TypeRef,
               access : ObjectEntryAccess,
               visibility : Visibility) -> Self {
        Self {
            name,
            description,
            unit,
            id,
            ty,
            access,
            visibility,
            node : OnceLock::new(),
        }
    }
    pub fn id(&self) -> u32 {
        self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn description(&self) -> Option<&str> {
        match &self.description {
            Some(some) => Some(&some),
            None => None,
        }
    }
    pub fn ty(&self) -> &TypeRef {
        &self.ty
    }
    pub fn access(&self) -> &ObjectEntryAccess {
        &self.access
    }
    pub fn unit(&self) -> Option<&str> {
        match &self.unit {
            Some(unit) => Some(&unit),
            None => None,
        }
    }
    pub fn __set_node(&self, node : NodeRef){
        self.node.set(node).expect("can't set the node of a object entry");
    }
    pub fn node(&self) -> &NodeRef {
        self.node.get().unwrap()
    }
}
