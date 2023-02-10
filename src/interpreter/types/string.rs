use std::sync::{Arc, RwLock};

use crate::interpreter::scopes::FieldRef;

use super::{MObjectImpl, MObject, builtin::BUILTINS, MFuncResult, MTypeRef, MObjectRef};
use delegate::delegate;

pub trait MString: MObject {}
pub type MStringRef = Arc<RwLock<dyn MString>>;

pub struct MStringImpl {
    mobject: MObjectImpl,
    value: String,
}
pub type MStringImplRef = Arc<RwLock<MStringImpl>>;
impl MObject for MStringImpl {
    delegate! {
        to self.mobject {
            fn get_field(&self, name: &str) -> Option<FieldRef>;
            fn insert_field(&self, field: FieldRef);
        }
    }
    fn objtype(&self) -> MTypeRef {
        BUILTINS.get_type("str")
    }
    fn str_nice(&self) -> MFuncResult {
        Ok(MStringImpl::from(self.value).wrap())
    }
    fn str_debug(&self) -> MFuncResult{
        Ok(MStringImpl::from(format!("'{}'", self.value)).wrap())
    }
    fn to_ext_string(&self, depth: usize, use_debug: bool) -> Result<String, MObjectRef> {
        Ok(self.value.to_owned())
    }
            
}
impl MString for MStringImpl {}

impl MStringImpl {
    pub fn new(value: String) -> Self {
        MStringImpl { mobject: MObjectImpl::new(), value }
    }
    pub fn wrap(self) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(self))
    }
}
impl From<MStringImpl> for MStringImplRef {
}
impl From<MStringImpl> for MStringRef {
}


// fn dot(
//     &self,
//     identifier: &str,
// ) -> Result<Arc<RwLock<dyn MshValue>>, Arc<RwLock<dyn MshValue>>> {
//     match identifier {
//         "len" => Ok(Arc::new(RwLock::new(MshInt::new(
//             (self.content.len() as usize).try_into().unwrap(),
//         )))),
//         _ => Err(Arc::new(RwLock::new(MshBaseError::new(&format!(
//             "member not found: `{}`",
//             identifier
//         ))))),
//     }
// }
impl From<String> for MStringImpl {
    fn from(c:String) -> Self {
        Self::new(c)
    }
}
impl From<&String> for MStringImpl {
    fn from(c: &String) -> Self {
        Self::new(c.to_owned())
    }
}
impl From<&str> for MStringImpl {
    fn from(c: &str) -> Self {
        Self::from(c.to_owned())
    }
}
