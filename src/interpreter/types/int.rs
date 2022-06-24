use crate::interpreter::scopes::AnnotatedField;

use super::{string::MshString, BinaryOperator, MshValue, MshReference};
use std::{
    fmt::Debug,
    sync::{Arc, RwLock},
};

#[derive(Debug)]
pub struct MshInt {
    pub value: isize,
}
impl MshInt {
    pub fn new(value: isize) -> Self {
        Self { value }
    }
}
impl MshValue for MshInt {
    fn objtype(&self) -> MshReference {
        MshString::from("int").into()
    }

    fn str_debug(&self) -> Result<MshReference, MshReference> {
        Ok(MshString::from(self.value.to_string()).into())
    }

    fn dot(&self, identifier: &str) -> Result<Option<Arc<RwLock<dyn AnnotatedField>>>, MshReference> {
        todo!()
    }
}
