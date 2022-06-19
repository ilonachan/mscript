use std::{
    error::Error,
    fmt::Display,
    sync::{Arc, RwLock},
};

use super::{string::MshString, MshValue};

pub trait MshError: MshValue + Error {}

#[derive(Debug)]
pub struct MshBaseError {
    msg: String,
}
impl MshBaseError {
    pub fn new(msg: &str) -> Self {
        Self {
            msg: msg.to_owned(),
        }
    }
}
impl Display for MshBaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}
impl Error for MshBaseError {}
impl MshValue for MshBaseError {
    fn objtype(&self) -> Arc<RwLock<dyn MshValue>> {
        Arc::new(RwLock::new(MshString::from("Error")))
    }

    fn to_string(&self) -> Result<MshString, Arc<RwLock<dyn MshValue>>> {
        Ok(MshString::from(format!(
            "{}: {}",
            self.objtype().read().unwrap().to_ext_string()?,
            self.msg
        )))
    }

    fn dot(
        &self,
        identifier: &str,
    ) -> Result<Arc<RwLock<dyn MshValue>>, Arc<RwLock<dyn MshValue>>> {
        todo!()
    }
}
impl MshError for MshBaseError {}
