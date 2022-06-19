use lazy_static::lazy_static;
use std::{
    fmt::Display,
    sync::{Arc, RwLock},
};

use super::{error::MshBaseError, string::MshString, MshValue};

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
    fn objtype(&self) -> Arc<RwLock<dyn MshValue>> {
        Arc::new(RwLock::new(MshString::from("none")))
    }

    fn to_string(&self) -> Result<MshString, Arc<RwLock<dyn MshValue>>> {
        Ok(MshString::from("none".to_owned()))
    }

    fn dot(
        &self,
        _identifier: &str,
    ) -> Result<Arc<RwLock<dyn MshValue>>, Arc<RwLock<dyn MshValue>>> {
        Err(Arc::new(RwLock::new(MshBaseError::new(
            "cannot access members of `none`",
        ))))
    }
}
