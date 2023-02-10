use std::{
    error::Error,
    fmt::Display,
    sync::{Arc, RwLock},
};

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
impl From<MshBaseError> for MshReference {
    fn from(o: MshBaseError) -> Self {
        msh_value_wrap(o)
    }
}

impl Display for MshBaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}
impl Error for MshBaseError {}
impl MshValue for MshBaseError {
    fn objtype(&self) -> MshReference {
        Arc::new(RwLock::new(MshString::from("Error")))
    }

    fn str_nice(&self) -> Result<MshReference, MshReference> {
        Ok(MshString::from(format!(
            "{}: {}",
            self.objtype().to_string_nice(0)?,
            self.msg
        )).into())
    }

    fn dot(&self, identifier: &str) -> Result<Option<Arc<RwLock<dyn crate::interpreter::scopes::AnnotatedField>>>, MshReference> {
        todo!()
    }
}
impl MshError for MshBaseError {}
