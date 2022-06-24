use lazy_static::lazy_static;
use std::{
    fmt::Display,
    sync::{Arc, RwLock},
};

use crate::interpreter::scopes::AnnotatedField;

use super::{error::MshBaseError, string::MshString, MshValue, MshReference, msh_value_wrap};

lazy_static! {
    // There is only one `none` value, its reference is shared globally.
    static ref MSH_NONE: Arc<RwLock<MshNoneValue>> = Arc::new(RwLock::new(MshNoneValue {}));
}
/// The language's `none` value is a singleton.
#[derive(Debug)]
pub struct MshNoneValue {}
impl MshNoneValue {
    pub fn get() -> Arc<RwLock<Self>> {
        MSH_NONE.clone()
    }
}
impl Display for MshNoneValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "none")
    }
}
impl MshValue for MshNoneValue {
    fn objtype(&self) -> MshReference {
        msh_value_wrap(MshString::from("none"))
    }

    fn str_nice(&self) -> Result<MshReference, MshReference> {
        Ok(msh_value_wrap(MshString::from("none".to_owned())))
    }

    fn dot(&self, identifier: &str) -> Result<Option<Arc<RwLock<dyn AnnotatedField>>>, MshReference> {
        Err(msh_value_wrap(MshBaseError::new("cannot access members of `none`")))
    }
}
