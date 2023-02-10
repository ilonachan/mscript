use std::{collections::HashMap, sync::{Arc, RwLock}};

use crate::{interpreter::scopes::{FieldRef, DynamicField}};

use super::{MTypeRef, MFuncResult, string::MString, MTypeImpl};

use delegate::delegate;

static MAX_EXTSTR_DEPTH: usize = 8;

pub trait MObject {
    ///Return the object's type in the Mscript type system. This type is itself an Mobject of type `type`.
    fn objtype(&self) -> MTypeRef;
    /**
    Format the object as a nice string for output printing. This SHOULD return an `MshString`, but
    if the user returns an object itself (which can itself be stringified) then that's fine as well.
    The actual acquiring of the string result happens in 
     */
    fn str_nice(&self) -> MFuncResult { self.str_debug() }
    /**
    Format the object as a nice string for debug printing.

    TODO: I want to force this to return a string, but a user implementing `$str` may have returned a value instead
    which CAN be turned into a string. So in the end I should probably just go down the rabbit hole when extracting the value.
     */
    fn str_debug(&self) -> MFuncResult {
        Ok(MString::from(format!(
            "object {:p} of type `{}`",
            &self,
            self.objtype().read().unwrap().name())).wrap())
        
    }
    /// convenience wrapper for `str_nice` to be used in Rust code.
    /// If the object isn't of type `string`, this method will recurse `MAX_EXTSTR_DEPTH` times
    /// before throwing an error
    fn to_ext_string(&self, depth: usize, use_debug: bool) -> Result<String, MObjectRef> {
        if depth > MAX_EXTSTR_DEPTH {
            // Err(MshBaseError::new("Error encoding object as `$str`: maximum recursion depth exceeded").wrap())?
        }
        if use_debug {
            self.str_debug()?.to_string_debug(depth+1)
        } else {
            self.str_nice()?.to_string_nice(depth+1)
        }
    }
    fn get_field(&self, name: &str) -> Option<FieldRef>;
    fn insert_field(&self, field: FieldRef);

    // TODO: add the functions that should be callable from Rust code on any object
}
// TODO: make this a macro
pub type MObjectRef = Arc<RwLock<dyn MObject>>;
impl MObject for MObjectRef {
    delegate! {
        to self.read().unwrap() {
            fn objtype(&self) -> MTypeRef;
            fn str_nice(&self) -> MFuncResult;
            fn str_debug(&self) -> MFuncResult;
            fn to_ext_string(&self, depth: usize, use_debug: bool) -> Result<String, MObjectRef>;
            fn get_field(&self, name: &str) -> Option<FieldRef>;
            fn insert_field(&self, field: FieldRef);
        }
    }
}

pub struct MObjectImpl {
    objtype: MTypeRef,
    inst_dict: HashMap<String, DynamicField>,
}
pub type MObjectImplRef = Arc<RwLock<MObjectImpl>>;
impl MObject for MObjectImpl {}
impl From<MObjectImpl> for MObjectRef {
    fn from(o: MObjectImpl) -> Self {
        o.wrap()
    }
}
impl From<MObjectImpl> for MObjectImplRef {
    fn from(o: MObjectImpl) -> Self {
        o.wrap()
    }
}
impl MObjectImpl {
    pub fn wrap(self) -> MObjectImplRef {
        Arc::new(RwLock::new(self))
    }
}


pub(super) fn create_object_type() -> MTypeRef {
    let _type = MTypeImpl::new("obj",None,vec![]).wrap();
    _type
}