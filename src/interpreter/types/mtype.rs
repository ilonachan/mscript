use std::{
    collections::HashMap,
    mem,
    sync::{Arc, RwLock},
};

pub type MFuncResult = Result<MObjectRef, MObjectRef>;

pub trait Field {
    fn name(&self) -> String;
    fn docstring(&self) -> Option<String>;
    fn can_read(&self) -> bool;
    fn can_write(&self) -> bool;
    fn get(&self) -> MFuncResult;
    fn set(&self, new_value: MObjectRef) -> MFuncResult;
}

pub struct ObjField {
    name: String,
    docstring: Option<String>,
    get: Option<impl Fn() -> MFuncResult>,
    set: Option<impl Fn(MObjectRef) -> MFuncResult>,
}
impl ObjField {
    pub fn new(
        name: String,
        docstring: Option<String>,
        get: Option<impl Fn() -> MFuncResult>,
        set: Option<impl Fn(MObjectRef) -> MFuncResult>,
    ) {
        ObjField {
            name,
            docstring,
            get,
            set,
        }
    }
}
impl Field for ObjField {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn docstring(&self) -> Option<String> {
        self.docstring.clone()
    }
    fn can_read(&self) -> bool {
        self.get.is_some()
    }
    fn can_write(&self) -> bool {
        self.set.is_some()
    }
    fn get(&self) -> MFuncResult {
        self.get()
    }
    fn set(&self, new_value: MObjectRef) -> MFuncResult {
        self.set(new_value)
    }
}

pub trait MObject {}
pub type MObjectRef = Arc<RwLock<dyn MObject>>;
pub struct MObjectImpl {
    objtype: MTypeRef,
    inst_dict: HashMap<String, ObjField>,
}
pub type MObjectImplRef = Arc<RwLock<MObjectImpl>>;
impl MObject for MObjectImpl {}
impl Into<MTypeRef> for MTypeImpl {
    fn into(self) -> MTypeRef {
        self.wrap()
    }
}
impl Into<MTypeImplRef> for MTypeImpl {
    fn into(self) -> MTypeImplRef {
        self.wrap()
    }
}

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
    proto_dict: HashMap<String, ObjField>,
    /// unfortunately, because the type hierarchy is a real mess at the top, the implementation of
    /// object functionality needs to be redone here.
    inst_dict: HashMap<String, ObjField>,
}
pub type MTypeImplRef = Arc<RwLock<MTypeImpl>>;
impl MObject for MTypeImpl {}

impl MType for MTypeImpl {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn supertypes(&self) -> &Vec<MTypeRef> {
        &self.supertypes
    }
}
impl Into<MTypeRef> for MTypeImpl {
    fn into(self) -> MTypeRef {
        self.wrap()
    }
}
impl Into<MTypeImplRef> for MTypeImpl {
    fn into(self) -> MTypeImplRef {
        self.wrap()
    }
}

impl MTypeImpl {
    pub fn wrap(self) -> Arc<RwLock<MTypeImpl>> {
        Arc::new(RwLock::new(self))
    }
}

fn init_base_types() -> (MTypeRef, MTypeRef) {
    let type_type = MTypeImpl {
        name: "type".into(),
        objtype: None,
        supertypes: vec![],
        proto_dict: HashMap::new(),
        inst_dict: HashMap::new(),
    }
    .wrap();
    let object_type = create_object_type(type_type.clone());
    type_type
        .write()
        .unwrap()
        .supertypes
        .push(object_type.clone());
    type_type.write().unwrap().objtype = Some(type_type.clone());
    (type_type, object_type)
}
fn create_object_type(type_type: MTypeRef) -> MTypeRef {
    let object_type = MTypeImpl {
        name: "obj".into(),
        objtype: Some(type_type),
        supertypes: vec![],
        proto_dict: HashMap::new(),
        inst_dict: HashMap::new(),
    }
    .wrap();
    object_type
}