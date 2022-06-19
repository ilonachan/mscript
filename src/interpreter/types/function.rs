use std::{collections::HashMap, sync::{Arc, RwLock}};

use crate::interpreter::stackmachine::{Statement, StackMachine, VarScope};

use super::{MshValue, string::MshString};

type ArgType = Arc<RwLock<dyn MshValue>>;

#[derive(Debug)]
pub struct MshFunction {
  pub name: String,
  pub doc: String,
  pub args: Vec<Option<ArgType>>,
  pub kwargs: HashMap<String, Option<ArgType>>,
  pub ret: Option<ArgType>,
  pub instructions: Vec<Statement>
}
impl MshValue for MshFunction {
    fn objtype(&self) -> Arc<RwLock<dyn MshValue>> {
        Arc::new(RwLock::new(MshString::from("func")))
    }

    fn dot(&self, identifier: &str) -> Result<Arc<RwLock<dyn MshValue>>, Arc<RwLock<dyn MshValue>>> {
        todo!()
    }

    fn call(
            &self,
            args: Vec<Arc<RwLock<dyn MshValue>>>,
            kwargs: HashMap<String, Arc<RwLock<dyn MshValue>>>,
            scope: std::sync::Arc<std::sync::RwLock<crate::interpreter::stackmachine::VarScope>>,
        ) -> Result<Arc<RwLock<dyn MshValue>>, Arc<RwLock<dyn MshValue>>> {
        StackMachine::exec(&self.instructions, Arc::new(RwLock::new(VarScope::new_local(scope.clone(), false))))
    }
}