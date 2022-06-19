pub mod string;
pub mod error;
pub mod int;
pub mod none;
pub mod function;

use core::any::Any;

use std::{
    collections::HashMap, sync::{Arc, RwLock}, fmt::Debug
};

use self::{error::MshBaseError, string::MshString};

use super::stackmachine::VarScope;

pub type MshReference<T> = Arc<RwLock<T>>;

pub enum UnaryOperator {
    Not, Bitnot,
    Inc, Dec
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BinaryOperator {
    And, Or, BitAnd, BitOr, Xor,
    AtOperator, Pow,
    Mul, Div, Mod,
    Plus, Minus
}

pub trait MshValue: Any + Debug {
    /**
    Return the object's type in the Mscript type system. This type is itself an Mobject of type `type`.

    TODO: restrict the output to types (if that's even a sensible thing to do)
    */
    fn objtype(&self) -> Arc<RwLock<dyn MshValue>>;

    /**
    Format the object as a string.
    FIXME: what's going on with this return signature?!
     */
    fn to_string(&self) -> Result<MshString, Arc<RwLock<dyn MshValue>>> {
        Ok(MshString::from(format!(
            "object {:p} of type `{}`",
            &self,
            self.objtype().read().unwrap().to_ext_string()?
        )))
    }

    fn to_ext_string(&self) -> Result<String, Arc<RwLock<dyn MshValue>>> {
        self.to_string().map(|o| o.to_ext_string()
            .unwrap_or_else(|_| panic!("the to_ext_string implementation of MshString is supposed to be reliable")))
    }

    /**
    execute the dot operator on the object, i.e. access a member.
     */
    fn dot(&self, identifier: &str) -> Result<Arc<RwLock<dyn MshValue>>, Arc<RwLock<dyn MshValue>>>;
    /**
    overwrite a member of the object according to the implemented dotset operator.
     */
    fn dotset(&self, _identifier: &str, _value: Arc<RwLock<dyn MshValue>>) -> Result<(), Arc<RwLock<dyn MshValue>>> {

        Err(Arc::new(RwLock::new(MshBaseError::new(&format!("cannot set members of `{}`", self.objtype().read().unwrap().to_ext_string().unwrap())))))
    }

    /**
    execute the indexing operator on the object.
     */
    fn index(&self, _index: Arc<RwLock<dyn MshValue>>) -> Result<Arc<RwLock<dyn MshValue>>, Arc<RwLock<dyn MshValue>>> {
        Err(Arc::new(RwLock::new(MshBaseError::new(&format!("cannot access index of `{}`", self.objtype().read().unwrap().to_ext_string().unwrap())))))
    }
    /**
    set the object's value at a specified index, using the indexing set operator.
     */
    fn indexset(&self, _index: Arc<RwLock<dyn MshValue>>, _value: Arc<RwLock<dyn MshValue>>) -> Result<(), Arc<RwLock<dyn MshValue>>> {
        Err(Arc::new(RwLock::new(MshBaseError::new(&format!("cannot set index of `{}`", self.objtype().read().unwrap().to_ext_string().unwrap())))))
    }

    /**
    Apply the specified binary operator.
     */
    fn binop(&self, _other: Arc<RwLock<dyn MshValue>>, _operator: BinaryOperator) -> Result<Arc<RwLock<dyn MshValue>>, Arc<RwLock<dyn MshValue>>> {
        Err(Arc::new(RwLock::new(MshBaseError::new(&format!("cannot operate on `{}`", self.objtype().read().unwrap().to_ext_string().unwrap())))))
    }
    /**
    Apply the selected unary operator. The first return value is the result of the expression, and the second
    is the value to be passed to the variable for an impure operator. (if `None` the first value is used)

    This is relevant for Post-Incrementing/Decrementing, because these operations may make in-place changes,
    but the previous value is still required. So a custom copying mechanism can be specified.
     */
    fn unop(&self, _operator: UnaryOperator) -> Result<(Arc<RwLock<dyn MshValue>>, Option<Arc<RwLock<dyn MshValue>>>), Arc<RwLock<dyn MshValue>>> {
        Err(Arc::new(RwLock::new(MshBaseError::new(&format!("cannot operate on `{}`", self.objtype().read().unwrap().to_ext_string().unwrap())))))
    }

    /**
    Calls the specified object as a function, with the specified positional and keyword arguments. The current scope
    of the program is specified so a function can invoke the Mscript interpreter to do its work,
    allowing Mscript functions to be called.
     */
    fn call(
        &self,
        _args: Vec<Arc<RwLock<dyn MshValue>>>,
        _kwargs: HashMap<String, Arc<RwLock<dyn MshValue>>>,
        _scope: Arc<RwLock<VarScope>>,
    ) -> Result<Arc<RwLock<dyn MshValue>>, Arc<RwLock<dyn MshValue>>> {
        Err(Arc::new(RwLock::new(MshBaseError::new(&format!("cannot call `{}`", self.objtype().read().unwrap().to_ext_string().unwrap())))))
    }

    // fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    //     write!(f, "{}", f.to_ext_string().unwrap_or_else(|e| format!("<ERROR calculating $str: {:?}>", e)))
    // }
}

pub trait MshType: MshValue {}