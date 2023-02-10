pub mod error;
pub mod none;
pub mod string;
pub mod builtin;
pub mod object;

use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, RwLock},
};

use self::{builtin::BUILTINS, object::{MObjectRef, MObject}};

use super::scopes::DynamicField;

pub type MFuncResult = Result<MObjectRef, MObjectRef>;

pub trait MType: MObject {
    fn name(&self) -> String;
    fn supertypes(&self) -> &Vec<MTypeRef>;
}
pub type MTypeRef = Arc<RwLock<dyn MType>>;
pub struct MTypeImpl {
    name: String,
    objtype: Option<MTypeRef>,
    supertypes: Vec<MTypeRef>,
    /// when a variable is not defined for a specific instance of an object,
    /// the object's type's `proto_dict` is consulted.
    proto_dict: HashMap<String, DynamicField>,
    /// unfortunately, because the type hierarchy is a real mess at the top, the implementation of
    /// object functionality needs to be redone here.
    inst_dict: HashMap<String, DynamicField>,
}
pub type MTypeImplRef = Arc<RwLock<MTypeImpl>>;
impl MObject for MTypeImpl {
    fn objtype(&self) -> MTypeRef {
        self.objtype.unwrap_or(BUILTINS.get_type("type"))
    }
}
impl MType for MTypeImpl {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn supertypes(&self) -> &Vec<MTypeRef> {
        &self.supertypes
    }
}
impl From<MTypeImpl> for MTypeRef {
    fn from(o: MTypeImpl) -> Self {
        o.wrap()
    }
}
impl From<MTypeImpl> for MTypeImplRef {
    fn from(o: MTypeImpl) -> Self {
        o.wrap()
    }
}

impl MTypeImpl {
    pub fn new(name: &str, objtype: Option<MTypeRef>, supertypes: Vec<MTypeRef>) -> MTypeImpl {
        MTypeImpl { name, objtype, supertypes, proto_dict: HashMap::new(), inst_dict: HashMap::new() }
    }
    pub fn wrap(self) -> Arc<RwLock<MTypeImpl>> {
        Arc::new(RwLock::new(self))
    }
}

/// list of unary operators defined in mscript
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum UnaryOperator {
    Not,
    Bitnot,
    Inc,
    Dec,
}
/// list of binary operators defined in mscript
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BinaryOperator {
    And,
    Or,
    BitAnd,
    BitOr,
    Xor,
    AtOperator,
    Pow,
    Mul,
    Div,
    Mod,
    Plus,
    Minus,
}