use std::sync::{Arc, RwLock};

use crate::interpreter::scopes::AnnotatedField;

use super::{msh_value_wrap, MshReference, MshValue};

#[derive(Debug)]
pub struct MshString {
    content: String,
}
impl MshString {
    pub fn new(content: String) -> Self {
        MshString { content }
    }
}
impl From<MshString> for MshReference {
    fn from(o: MshString) -> Self {
        msh_value_wrap(o)
    }
}

impl MshValue for MshString {
    fn objtype(&self) -> MshReference {
        MshString::from("str").into()
    }

    fn str_nice(&self) -> Result<MshReference, MshReference> {
        Ok(MshString::from(&self.content).into())
    }

    fn str_debug(&self) -> Result<MshReference, MshReference> {
        Ok(MshString::from(format!("'{}'", self.content)).into())
    }

    fn to_string_nice(&self, _depth: usize) -> Result<String, MshReference> {
        Ok(self.content.to_owned())
    }
    fn to_string_debug(&self, _depth: usize) -> Result<String, MshReference> {
        Ok(format!("'{}'",self.content))
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

    fn index(&self, index: MshReference) -> Result<Option<Arc<RwLock<dyn AnnotatedField>>>, MshReference> {
        // TODO: use my own type system instead
        // if index.type_id() == TypeId::of::<MshInt>() {
        //     let index = (index as Box<dyn Any>).downcast::<MshInt>().unwrap();
        //     Ok(Arc::new(RwLock::new(MshString::from(format!( "{}", index.value )))))
        // } else {
        //     Err(Arc::new(RwLock::new(MshBaseError::new("invalid index type"))))
        // }
        todo!()
    }

    fn dot(&self, identifier: &str) -> Result<Option<Arc<RwLock<dyn AnnotatedField>>>, MshReference> {
        todo!()
    }
}
impl From<String> for MshString {
    fn from(c:String) -> Self {
        Self::new(c)
    }
}
impl From<&String> for MshString {
    fn from(c: &String) -> Self {
        Self::new(c.to_owned())
    }
}
impl From<&str> for MshString {
    fn from(c: &str) -> Self {
        Self::from(c.to_owned())
    }
}
