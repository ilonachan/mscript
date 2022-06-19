use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::interpreter::stackmachine::VarScope;

use super::{error::MshBaseError, int::MshInt, BinaryOperator, MshValue};

#[derive(Debug)]
pub struct MshString {
    content: String,
}
impl MshString {
    pub fn new(content: &str) -> Self {
        MshString {
            content: content.to_owned(),
        }
    }
}
impl MshValue for MshString {
    fn objtype(&self) -> Arc<RwLock<dyn MshValue>> {
        Arc::new(RwLock::new(MshString::from("str")))
    }

    fn to_string(&self) -> Result<MshString, Arc<RwLock<dyn MshValue>>> {
        Ok(MshString::from(&self.content))
    }

    fn to_ext_string(&self) -> Result<String, Arc<RwLock<dyn MshValue>>> {
        Ok(self.content.to_owned())
    }

    fn dot(
        &self,
        identifier: &str,
    ) -> Result<Arc<RwLock<dyn MshValue>>, Arc<RwLock<dyn MshValue>>> {
        match identifier {
            "len" => Ok(Arc::new(RwLock::new(MshInt::new(
                (self.content.len() as usize).try_into().unwrap(),
            )))),
            _ => Err(Arc::new(RwLock::new(MshBaseError::new(&format!(
                "member not found: `{}`",
                identifier
            ))))),
        }
    }

    fn index(
        &self,
        index: Arc<RwLock<dyn MshValue>>,
    ) -> Result<Arc<RwLock<dyn MshValue>>, Arc<RwLock<dyn MshValue>>> {
        // TODO: use my own type system instead
        // if index.type_id() == TypeId::of::<MshInt>() {
        //     let index = (index as Box<dyn Any>).downcast::<MshInt>().unwrap();
        //     Ok(Arc::new(RwLock::new(MshString::from(format!( "{}", index.value )))))
        // } else {
        //     Err(Arc::new(RwLock::new(MshBaseError::new("invalid index type"))))
        // }
        todo!()
    }

    fn binop(
        &self,
        other: Arc<RwLock<dyn MshValue>>,
        operator: BinaryOperator,
    ) -> Result<Arc<RwLock<dyn MshValue>>, Arc<RwLock<dyn MshValue>>> {
        todo!()
    }

    fn call(
        &self,
        args: Vec<Arc<RwLock<dyn MshValue>>>,
        kwargs: HashMap<String, Arc<RwLock<dyn MshValue>>>,
        scope: Arc<RwLock<VarScope>>,
    ) -> Result<Arc<RwLock<dyn MshValue>>, Arc<RwLock<dyn MshValue>>> {
        todo!()
    }
}
impl From<&str> for MshString {
    fn from(c: &str) -> Self {
        Self::new(c)
    }
}
impl From<&String> for MshString {
    fn from(c: &String) -> Self {
        Self::new(&c[..])
    }
}
impl From<String> for MshString {
    fn from(c: String) -> Self {
        Self::from(&c[..])
    }
}
