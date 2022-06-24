use std::{fmt::Debug, sync::{Arc, RwLock}};

use crate::interpreter::scopes::{VarScope, AnnotatedField};

use super::{MshValue, MshReference, error::MshBaseError, string::MshString};



pub struct MshObject {
  objtype: Option<MshReference>,
  own_scope: VarScope,
}
impl MshObject {
  fn instance(objtype: Option<MshReference>) -> Self {
    // MshObject { objtype, own_scope: VarScope::new_local(objtype.proto_dict, false) }
    MshObject { objtype, own_scope: VarScope::new_global(false) }
  }
  // }
}
impl Debug for MshObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string_debug(0).or(Err(std::fmt::Error))?)
    }
}
impl MshValue for MshObject {
    fn objtype(&self) -> MshReference {
        self.objtype.as_ref().unwrap().clone()
    }

    fn dot(&self, identifier: &str) -> Result<Option<Arc<RwLock<dyn AnnotatedField>>>, MshReference> {
        todo!()
    }

    fn str_nice(&self) -> Result<MshReference, MshReference> {
        self.str_debug()
    }

    fn str_debug(&self) -> Result<MshReference, MshReference> {
        Ok(MshString::from(format!(
            "object {:p} of type `{}`",
            &self,
            self.objtype().read().unwrap().to_string_nice(0)?
        )).into())
    }

    fn index(&self, index: MshReference) -> Result<Option<Arc<RwLock<dyn AnnotatedField>>>, MshReference> {
        Err(MshBaseError::new(&format!(
            "cannot access index of `{}`",
            self.objtype().to_string_nice(0).unwrap()
        )).into())
    }
}