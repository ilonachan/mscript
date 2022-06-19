use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use super::{types::{error::MshBaseError, none::MshNoneValue, BinaryOperator, MshValue, string::MshString}};
use crate::parser::mshparser::MshParserContext;

#[derive(Debug)]
pub enum Statement {
    LoadStatic(Arc<RwLock<dyn MshValue>>),
    BinOperator(BinaryOperator),
    LoadScope(String),
    LoadGlobal(String),
    StoreScope(String),
    StoreGlobal(String),
    Dot(String),
    Index,
    Call,
}

pub enum ScopeReference {
    /// The variable is locally declared but not yet assigned.
    Local,
    /// The variable is locally declared and assigned.
    LocalValue(Arc<RwLock<dyn MshValue>>),
    /// The variable explicitly defers to the global scope.
    Global,
    /// The variable isn't defined in this scope, so any access implicitly propagates.
    /// (not used internally)
    Propagate,
}

/// A container representation of variable scopes, for use in the interpreter AND a potential typechecker.
pub struct VarScope {
    /// this is only zero for the global scope itself
    pub global: Option<Arc<RwLock<Self>>>,
    /// any non-global scope must have a parent, this can also be the global scope
    parent: Option<Arc<RwLock<Self>>>,
    /// the values consist of a behavior specifier (and usually an encapsulated value),
    /// combined with an optional type hint.
    values: HashMap<String, (ScopeReference, Option<Arc<RwLock<dyn MshValue>>>)>,
    /// the scope needs to be informed of the `#!strict assign` setting in the interpreter
    pub strict_assign: bool,
}

impl VarScope {
    pub fn find_global_scope(scope: Arc<RwLock<Self>>) -> Arc<RwLock<Self>> {
        scope
            .as_ref()
            .read()
            .unwrap()
            .global
            .clone()
            .unwrap_or(scope.clone())
    }

    /// Create a new global scope as a root for the scope tree
    pub fn new_global(strict_assign: bool) -> Self {
        VarScope {
            global: None,
            parent: None,
            values: HashMap::new(),
            strict_assign,
        }
    }
    /**
    Create a new child scope under the specified scope tree.

    Each local scope holds a shortcut reference to the global root scope,
    which is automatically obtained from the parent (i.e. reparenting a scope is not possible)
    */
    pub fn new_local(parent: Arc<RwLock<Self>>, strict_assign: bool) -> Self {
        VarScope {
            // if a global scope was specified, use that.
            // if not, reuse the paren't global scope (if present).
            global: Some(Self::find_global_scope(parent.clone())),
            parent: Some(parent.clone()),
            values: HashMap::new(),
            strict_assign,
        }
    }

    /**
    Get the value of a variable in the scope. If the variable isn't declared or defers to global,
    the call is propagated upwards in the scope tree.

    This function always returns a value according to the defined algorithm,
    and `None` even if a variable isn't declared. if raw information about
    the scope is needed, use `get_behavior`, `is_declared` or `has_value` instead.
    */
    pub fn get(&self, id: &str) -> Arc<RwLock<dyn MshValue>> {
        return match self.values.get(id) {
            Some((ScopeReference::LocalValue(o), _)) => o.clone(),
            // Propagate is never actually used, but a key not being present implicitly behaves the same way.
            Some((ScopeReference::Propagate, _)) | None => match &self.parent {
                Some(o) => o.as_ref().read().unwrap().get(id),
                None => MshNoneValue::get(),
            },
            Some((ScopeReference::Global, _)) => match &self.global {
                Some(o) => o.as_ref().read().unwrap().get(id),
                None => MshNoneValue::get(),
            },
            Some((ScopeReference::Local, _)) => MshNoneValue::get(),
        };
    }
    /**
    Indicates how a variable's value can be found. Possible values are:

    * `Local`: the variable is locally declared, assigned or not
    * `Global`: the variable explicitly defers to the global scope
    * `Propagate`: the variable isn't defined, so it implicitly defers up the scope tree

    Note that `LocalValue` is not a possible return value: because using `clone` to return the
    encapsulated value is costly, we simplify that option to `Local`.
    */
    pub fn get_behavior(&self, id: &str) -> ScopeReference {
        return match self.values.get(id) {
            Some((ScopeReference::LocalValue(_), _)) | Some((ScopeReference::Local, _)) => {
                ScopeReference::Local
            }
            // Propagate is never actually used, but a key not being present implicitly behaves the same way.
            Some((ScopeReference::Propagate, _)) | None => ScopeReference::Propagate,
            Some((ScopeReference::Global, _)) => ScopeReference::Global,
        };
    }
    /**
    Whether the variable is declared in the scope tree, according to the defined algorithm.
    Note that this function returns `true` if the variable isn't defined here but in a parent scope,
    and it returns `false` if the variable is declared as deferring to a nonexistent global variable.
     */
    pub fn is_declared(&self, id: &str) -> bool {
        return match self.values.get(id) {
            Some((ScopeReference::Local, _)) | Some((ScopeReference::LocalValue(_), _)) => true,
            Some((ScopeReference::Global, _)) => match &self.global {
                Some(o) => o.as_ref().read().unwrap().is_declared(id),
                None => false,
            },
            Some((ScopeReference::Propagate, _)) | None => match &self.parent {
                Some(o) => o.as_ref().read().unwrap().is_declared(id),
                None => false,
            },
        };
    }
    /**
    Whether the variable has a value in the scope tree. If `is_declared()==false` this is also `false`,
    but the converse isn't necessarily true: a variable can be initially declared without having received a value yet.
     */
    pub fn has_value(&self, id: &str) -> bool {
        return match self.values.get(id) {
            Some((ScopeReference::Local, _)) => false,
            Some((ScopeReference::LocalValue(_), _)) => true,
            Some((ScopeReference::Global, _)) => match &self.global {
                Some(o) => o.as_ref().read().unwrap().has_value(id),
                None => false,
            },
            Some((ScopeReference::Propagate, _)) | None => match &self.parent {
                Some(o) => o.as_ref().read().unwrap().has_value(id),
                None => false,
            },
        };
    }
    /**
    Get the type hint for a variable, if it has been declared with one.
    TODO: can global variables be declared with a type hint that differs from their global definition?
    It could make sense if global variables are always typeless, or if one can't rely on their type hints well...
     */
    pub fn get_type_hint(&self, id: &str) -> Option<Arc<RwLock<dyn MshValue>>> {
        return match self.values.get(id) {
            Some((ScopeReference::Local, o)) | Some((ScopeReference::LocalValue(_), o)) => {
                o.clone()
            }
            Some((ScopeReference::Global, o)) => o.clone().or(match &self.global {
                Some(o) => o.as_ref().read().unwrap().get_type_hint(id),
                None => None,
            }),
            Some((ScopeReference::Propagate, _)) | None => match &self.parent {
                Some(o) => o.as_ref().read().unwrap().get_type_hint(id),
                None => None,
            },
        };
    }

    /**
    Set the value of a variable in scope, deferring to the parent/global scope if necessary.

    If a variable isn't declared yet, it will be declared locally with the new value.
    However this only happens if `#!strict assign` isn't set; if it is the function simply returns `false`.
    It returns `true` iff the value was correctly assigned, which is true in every other case.
     */
    pub fn set(&mut self, id: &str, val: Arc<RwLock<dyn MshValue>>) -> bool {
        if self.is_declared(id) {
            match self.values.get(id) {
                Some((ScopeReference::LocalValue(_),_)) | Some((ScopeReference::Local,_)) => {
                    let o = self.values.remove(id).unwrap();
                    self.values.insert(id.to_owned(), (ScopeReference::LocalValue(val), o.1));
                },
                Some((ScopeReference::Global,_)) => match self.global {
                    Some(ref g) => return g.as_ref().write().unwrap().set(id, val),
                    None => panic!("the global scope isn't supposed to defer to Global")
                },
                Some((ScopeReference::Propagate,_)) | None => return self.parent.as_deref()
                    .expect("if a propagating variable is declared, then the parent must hold that declaration,
                    and in particular the parent must exist.").write().unwrap().set(id, val),
            };
        } else {
            if self.strict_assign {
                // TODO: respect #!strict assign, prevent the user from falling into this case
                return false;
            } else {
                self.declare(id, ScopeReference::LocalValue(val), None)
            }
        };
        return true;
    }
    /**
    Declare a new variable in the scope; can also redeclare existing variables,
    and undeclare variables if `Propagate` is specified. An optional type hint can be provided.
     */
    pub fn declare(
        &mut self,
        id: &str,
        value: ScopeReference,
        type_hint: Option<Arc<RwLock<dyn MshValue>>>,
    ) {
        match value {
            ScopeReference::Propagate => self.values.remove(id),
            _ => self.values.insert(id.to_owned(), (value, type_hint)),
        };
    }
}

pub struct StackMachine {}

impl StackMachine {
    pub fn exec(
        instructions: &Vec<Statement>,
        scope: Arc<RwLock<VarScope>>,
    ) -> Result<Arc<RwLock<dyn MshValue>>, Arc<RwLock<dyn MshValue>>> {
        let mut value_stack = Vec::<Arc<RwLock<dyn MshValue>>>::new();
        let global_scope = VarScope::find_global_scope(scope.clone());
        for inst in instructions {
            match inst {
                Statement::LoadStatic(val) => value_stack.push(val.clone()),
                Statement::BinOperator(op) => {
                    let b = value_stack.pop();
                    let a = value_stack.pop();
                    if let None = a {
                        Err(Arc::new(RwLock::new(MshBaseError::new(
                            "not enough arguments for binary operator",
                        ))) as Arc<RwLock<dyn MshValue>>)?;
                    }
                    let (a, b) = (a.unwrap(), b.unwrap());
                    value_stack.push(a.read().unwrap().binop(b, *op)?);
                }
                Statement::LoadScope(id) => value_stack.push(scope.read().unwrap().get(id)),
                Statement::LoadGlobal(id) => {
                    value_stack.push(global_scope.read().unwrap().get(id));
                }
                Statement::StoreScope(id) => {
                    scope.write().unwrap().set(
                        id,
                        value_stack
                            .pop()
                            .ok_or(Arc::new(RwLock::new(MshBaseError::new(
                                "no value to assign",
                            ))) as Arc<RwLock<dyn MshValue>>)?,
                    );
                }
                Statement::StoreGlobal(id) => {
                    global_scope.write().unwrap().set(
                        id,
                        value_stack
                            .pop()
                            .ok_or(Arc::new(RwLock::new(MshBaseError::new(
                                "no value to assign",
                            ))) as Arc<RwLock<dyn MshValue>>)?,
                    );
                }
                Statement::Dot(id) => {
                    let a = value_stack.pop();
                    if let None = a {
                        Err(Arc::new(RwLock::new(MshBaseError::new(
                            "no value to index into",
                        ))) as Arc<RwLock<dyn MshValue>>)?;
                    }
                    value_stack.push(a.unwrap().read().unwrap().dot(id)?)
                }
                Statement::Index => todo!(),
                Statement::Call => todo!(),
            }
        }
        if value_stack.len() > 1 {
            Err(Arc::new(RwLock::new(MshBaseError::new(
                "too many return values",
            ))) as Arc<RwLock<dyn MshValue>>)?
        } else {
            Ok(value_stack.pop().unwrap_or(MshNoneValue::get()))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::interpreter::types::int::MshInt;

    use super::*;
    use std::vec;

    static STRICT_ASSIGN: bool = false;

    #[test]
    fn create_scopes() {
        let global_scope = Arc::new(RwLock::new(VarScope::new_global(STRICT_ASSIGN)));
        let scope1 = Arc::new(RwLock::new(VarScope::new_local(
            global_scope.clone(),
            STRICT_ASSIGN,
        )));
        let scope2 = Arc::new(RwLock::new(VarScope::new_local(
            scope1.clone(),
            STRICT_ASSIGN,
        )));

        assert!(!scope2.as_ref().read().unwrap().is_declared("test"));
        scope1.as_ref().write().unwrap().declare(
            "test",
            ScopeReference::LocalValue(MshNoneValue::get()),
            None,
        );
        assert!(scope2.as_ref().read().unwrap().is_declared("test"));
        // assert_eq!(scope2.as_ref().read().unwrap().get("test"), MshNoneValue::get());
        assert!(!global_scope.as_ref().read().unwrap().is_declared("test"));
    }

    #[test]
    fn interpreter() {
        let global_scope = Arc::new(RwLock::new(VarScope::new_global(STRICT_ASSIGN)));
        let local_scope = Arc::new(RwLock::new(VarScope::new_local(global_scope.clone(), STRICT_ASSIGN)));

        let instructions = vec![
            Statement::LoadStatic(Arc::new(RwLock::new(MshInt::new(42)))),
            Statement::StoreGlobal("test".to_owned()),
            Statement::LoadScope("test".to_owned()),
        ];
        let res = StackMachine::exec(&instructions, local_scope.clone());

        let local_scope = local_scope.read().unwrap();
        assert!(local_scope.is_declared("test"));
        // assert_eq!(local_scope.get("test").read().unwrap(), MshInt::new(42));
        // assert_eq!(res.unwrap_or_else(|| panic!()).read().unwrap(), MshInt::new(42));
    }
}
