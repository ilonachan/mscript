pub mod error;
// pub mod function;
pub mod int;
pub mod none;
pub mod string;
pub mod object;
pub mod mtype;

use core::any::Any;

use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, RwLock},
};

use delegate::delegate;

use self::{error::MshBaseError, string::MshString};

use super::scopes::{AnnotatedField, VarScope};

pub type MshReference = Arc<RwLock<dyn MshValue>>;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum UnaryOperator {
    Not,
    Bitnot,
    Inc,
    Dec,
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BinaryOperator {
    And,
    Or,
    BitAnd,
    BitOr,
    Xor,
    AtOperator,
    Pow,
    Mul,
    Div,
    Mod,
    Plus,
    Minus,
}

static MAX_EXTSTR_DEPTH: usize = 8;

#[allow(unused_variables)]
pub trait MshValue: Any + Debug {
    /**
    Return the object's type in the Mscript type system. This type is itself an Mobject of type `type`.

    TODO: restrict the output to types (if that's even a sensible thing to do)
    */
    fn objtype(&self) -> MshReference;

    /**
    Format the object as a nice string for output printing. This SHOULD return an `MshString`, but
    if the user returns an object itself (which can itself be stringified) then that's fine as well.
    The actual acquiring of the string result happens in 
     */
    fn str_nice(&self) -> Result<MshReference, MshReference> {
        self.str_debug()
    }

    /**
    Format the object as a nice string for debug printing.

    TODO: I want to force this to return a string, but a user implementing `$str` may have returned a value instead
    which CAN be turned into a string. So in the end I should probably just go down the rabbit hole when extracting the value.
     */
    fn str_debug(&self) -> Result<MshReference, MshReference> {
        Ok(MshString::from(format!(
            "object {:p} of type `{}`",
            &self,
            self.objtype().read().unwrap().to_string_nice(0)?
        )).into())
    }

    fn to_string_nice(&self, depth: usize) -> Result<String, MshReference> {
        if depth > MAX_EXTSTR_DEPTH {
            Err::<(), MshReference>(MshBaseError::new("Error encoding object as `$str`: maximum recursion depth exceeded").into())?
        }
        self.str_nice()?.to_string_nice(depth+1)
    }

    fn to_string_debug(&self, depth: usize) -> Result<String, MshReference> {
        if depth > MAX_EXTSTR_DEPTH {
            Err::<(), MshReference>(MshBaseError::new("Error encoding object as `$str`: maximum recursion depth exceeded").into())?
        }
        self.str_debug()?.to_string_debug(depth+1)
    }

    /**
    execute the dot operator on the object, i.e. access a member.
     */
    fn dot(&self, identifier: &str) -> Result<Option<Arc<RwLock<dyn AnnotatedField>>>, MshReference>;

    /**
    execute the indexing operator on the object.
     */
    fn index(&self, index: MshReference) -> Result<Option<Arc<RwLock<dyn AnnotatedField>>>, MshReference> {
        Err(MshBaseError::new(&format!(
            "cannot access index of `{}`",
            self.objtype().to_string_nice(0).unwrap()
        )).into())
    }

    /**
    Apply the specified binary operator.
     */
    fn binop(
        &self,
        other: MshReference,
        operator: BinaryOperator,
    ) -> Result<MshReference, MshReference> {
        Err(MshBaseError::new(&format!(
            "cannot operate on `{}`",
            self.objtype().to_string_nice(0).unwrap()
        )).into())
    }
    /**
    Apply the selected unary operator. The first return value is the result of the expression, and the second
    is the value to be passed to the variable for an impure operator. (if `None` the first value is used)

    This is relevant for Post-Incrementing/Decrementing, because these operations may make in-place changes,
    but the previous value is still required. So a custom copying mechanism can be specified.
     */
    fn unop(
        &self,
        operator: UnaryOperator,
    ) -> Result<
        (MshReference, Option<MshReference>),
        MshReference,
    > {
        Err(MshBaseError::new(&format!(
            "cannot operate on `{}`",
            self.objtype().to_string_nice(0).unwrap()
        )).into())
    }

    /**
    Calls the specified object as a function, with the specified positional and keyword arguments. The current scope
    of the program is specified so a function can invoke the Mscript interpreter to do its work,
    allowing Mscript functions to be called.
     */
    fn call(
        &self,
        args: Vec<MshReference>,
        kwargs: HashMap<String, MshReference>,
        scope: Arc<RwLock<VarScope>>,
    ) -> Result<MshReference, MshReference> {
        Err(MshBaseError::new(&format!(
            "cannot call `{}`",
            self.objtype().to_string_nice(0).unwrap()
        )).into())
    }

    // fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    //     write!(f, "{}", f.to_ext_string().unwrap_or_else(|e| format!("<ERROR calculating $str: {:?}>", e)))
    // }
}

impl MshValue for MshReference {
    delegate! {
        to self.read().unwrap() {
            fn objtype(&self) -> MshReference;
            fn str_nice(&self) -> Result<MshReference, MshReference>;
            fn str_debug(&self) -> Result<MshReference, MshReference>;
            fn to_string_nice(&self, depth: usize) -> Result<String, MshReference>;
            fn to_string_debug(&self, depth: usize) -> Result<String, MshReference>;
            fn call(
                &self,
                args: Vec<MshReference>,
                kwargs: HashMap<String, MshReference>,
                scope: Arc<RwLock<VarScope>>,
            ) -> Result<MshReference, MshReference>;
            fn dot(&self, identifier: &str) -> Result<Option<Arc<RwLock<dyn AnnotatedField>>>, MshReference>;
            fn index(&self, index: MshReference) -> Result<Option<Arc<RwLock<dyn AnnotatedField>>>, MshReference>;
            fn binop(
                &self,
                other: Arc<RwLock<dyn MshValue>>,
                operator: BinaryOperator,
            ) -> Result<Arc<RwLock<dyn MshValue>>, Arc<RwLock<dyn MshValue>>>;
            fn unop(
                &self,
                operator: UnaryOperator,
            ) -> Result<
                (Arc<RwLock<dyn MshValue>>, Option<Arc<RwLock<dyn MshValue>>>),
                Arc<RwLock<dyn MshValue>>,
            >;
        }
    }
}

pub(crate) fn msh_value_wrap(obj: impl MshValue) -> MshReference {
    Arc::new(RwLock::new(obj))
}

pub trait MshType: MshValue {}
