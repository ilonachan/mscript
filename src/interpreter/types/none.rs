use lazy_static::lazy_static;
use std::{
    sync::{Arc, RwLock},
};

use super::{object::{MObject, MObjectImpl}, MTypeImpl, builtin::BUILTINS, MTypeRef, string::MStringImpl, MFuncResult};

lazy_static! {
    // There is only one `none` value, its reference is shared globally.
    static ref MSH_NONE: MNoneRef = Arc::new(RwLock::new(MNone::singleton()));
}

struct MNone;
pub type MNoneRef = Arc<RwLock<MNone>>;
impl MObject for MNone {
    fn objtype(&self) -> MTypeRef {
        BUILTINS.get_type("none")
    }
    fn str_nice(&self) -> MFuncResult {
        Ok(MStringImpl::from("none").wrap())
    }
}

impl MNone {
    fn singleton() -> MNone {
        MObjectImpl::new(BUILTINS.get_type("none"))
    }
    pub fn refer() -> MNoneRef {
        MSH_NONE.clone()
    }
}

pub(super) fn create_none_type() -> MTypeRef {
    let _type = MTypeImpl::new("none", None, vec![BUILTINS.get_type("obj")]).wrap();
    _type
}