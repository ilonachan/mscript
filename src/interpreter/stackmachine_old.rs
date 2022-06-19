use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Debug)]
pub enum Statement {
    LoadStatic(StaticValue),
    BinOperator(String),
    LoadScope(String),
    LoadGlobal(String),
    StoreScope(String),
    StoreGlobal(String),
    Dot(String),
    Index,
    Call,
}

#[derive(Debug, PartialEq)]
pub enum ObjectValue {
    List(Vec<StaticValue>),
    // TODO: should the keys here support arbitrary objects? Potentially not, because
    Dict(HashMap<String, StaticValue>),
}

impl ObjectValue {
    pub fn to_string(&self) -> Result<String, StaticValue> {
        match self {
            ObjectValue::List(v) => match v.get(0) {
                None => Ok("[]".to_owned()),
                Some(s) => Ok("[".to_owned()
                    + &s.to_string()?
                    + &v.iter()
                        .try_fold(String::new(), |a, b| Ok(a + ", " + &b.to_string()?))?
                    + "]"),
            },
            ObjectValue::Dict(m) => match self.dot("$tostr")? {
                StaticValue::Function(f) => Ok(f),
                _ => Ok(format!(
                    "object {:p} of type {}",
                    &self,
                    self.objtype().to_string()?
                )),
            },
        }
    }
    pub fn objtype(&self) -> StaticValue {
        StaticValue::StringLiteral("obj".to_owned())
    }
    pub fn dot(&self, identifier: &str) -> Result<StaticValue, StaticValue> {
        Err(StaticValue::StringLiteral(format!(
            "identifier `{}` doesn't exist",
            identifier
        )))
    }
    pub fn index(&self, index: StaticValue) -> Result<StaticValue, StaticValue> {
        match self {
            ObjectValue::List(v) => {
                if let StaticValue::Int(i) = index {
                    if i >= 0 && let Some(val) = v.get(i as usize) {
                        Ok(val.clone())
                    } else {
                        Err(StaticValue::StringLiteral("index out of bounds".to_owned()))
                    }
                } else {
                    Err(StaticValue::StringLiteral("invalid index type".to_owned()))
                }
            }
            ObjectValue::Dict(m) => {
                if let Some(f) = m.get("$index") {
                    f.call(vec![index], HashMap::new())
                } else {
                    Err(StaticValue::StringLiteral(format!(
                        "indexing not supported for type `{}`",
                        self.objtype().to_string()?
                    )))
                }
            }
        }
    }
    pub fn operator(&self, other: StaticValue, operator: &str) -> Result<StaticValue, StaticValue> {
        Err(StaticValue::StringLiteral(format!(
            "operator `{}` not supported between types `{}`,`{}`",
            operator,
            self.objtype().to_string()?,
            other.objtype().to_string()?
        )))
    }
    pub fn call(
        &self,
        args: Vec<StaticValue>,
        kwargs: HashMap<String, StaticValue>,
    ) -> Option<Result<StaticValue, StaticValue>> {
        if let Self::Dict(m) = self {
            if let Some(f) = m.get("$call") {
                Some(f.call(args, kwargs))
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StaticValue {
    Int(i64),
    Float(f64),
    StringLiteral(String),
    Bool(bool),
    Object(Rc<RefCell<ObjectValue>>),
    Function(String),
    None,
}

impl StaticValue {
    /// Obtains a string representation of any kind of object, or throw an error
    pub fn to_string(&self) -> Result<String, StaticValue> {
        match self {
            StaticValue::Int(i) => Ok(i.to_string()),
            StaticValue::Float(f) => Ok(f.to_string()),
            StaticValue::StringLiteral(s) => Ok(s.to_owned()),
            StaticValue::Bool(b) => Ok({
                if *b {
                    "true"
                } else {
                    "false"
                }
            }
            .to_owned()),
            StaticValue::Object(o) => o.as_ref().borrow().to_string(),
            StaticValue::Function(fname) => Ok(format!("func {}() -> none", fname)),
            StaticValue::None => Ok("none".to_owned()),
        }
    }

    /// Obtains the type of an object
    ///
    /// TODO: Currently the types returned are just the string name of the type.
    ///       Construct type objects in the standard library and return those instead.
    pub fn objtype(&self) -> StaticValue {
        match self {
            Self::Int(_) => Self::StringLiteral("int".to_owned()),
            Self::Float(_) => Self::StringLiteral("float".to_owned()),
            Self::StringLiteral(_) => Self::StringLiteral("str".to_owned()),
            Self::Bool(_) => Self::StringLiteral("bool".to_owned()),
            Self::Object(o) => o.as_ref().borrow().objtype(),
            Self::Function(_f) => Self::StringLiteral("func".to_owned()),
            Self::None => Self::StringLiteral("none".to_owned()),
        }
    }
    /// Gets a subobject/method using the dot operator.
    /// Throws an error if the property doesn't exist, or rethrows if the user has
    /// overridden the dot behavior and that behavior throws.
    ///
    /// TODO: use an error object instead of a string literal
    /// TODO: actually implement these dot methods.
    pub fn dot(&self, identifier: &str) -> Result<StaticValue, StaticValue> {
        Err(StaticValue::StringLiteral(format!(
            "identifier {} doesn't exist",
            identifier
        )))
    }
    /// Gets a subobject using the indexing operator.
    /// If the type's implementation of the indexing behavior throws, that error is rethrown.
    ///
    /// TODO: actually implement indexing.
    pub fn index(&self, index: StaticValue) -> Result<StaticValue, StaticValue> {
        Ok(Self::None)
    }
    /// Applies a binary operator to `self` and `other`, in that order.
    /// Because this is a common pattern, this function handles all operators
    /// and distinguishes between them internally.
    ///
    /// TODO: actually implement operators
    /// TODO: use error objects
    pub fn operator(&self, other: StaticValue, operator: &str) -> Result<StaticValue, StaticValue> {
        Err(Self::StringLiteral(format!(
            "operator {} not supported between {},{}",
            operator,
            self.objtype().to_string()?,
            other.objtype().to_string()?
        )))
    }
    /// Tries to call an object. Throws if the object isn't callable,
    /// and obviously rethrows errors inside the call.
    ///
    /// TODO: use error objects
    /// TODO: functions aren't well developed
    pub fn call(
        &self,
        args: Vec<StaticValue>,
        kwargs: HashMap<String, StaticValue>,
    ) -> Result<StaticValue, StaticValue> {
        match self {
      Self::Function(f) => Ok(Self::None),
      Self::Object(o) if let Some(ret) = o.as_ref().borrow().call(args, kwargs) => ret,
      _ => Err(Self::StringLiteral(format!("type `{}` is not callable", self.objtype().to_string().unwrap()))),
    }
    }
}

pub enum ScopeReference {
    /// The variable is locally declared but not yet assigned.
    Local,
    /// The variable is locally declared and assigned.
    LocalValue(StaticValue),
    /// The variable explicitly defers to the global scope.
    Global,
    /// The variable isn't defined in this scope, so any access implicitly propagates.
    /// (not used internally)
    Propagate,
}

/// A container representation of variable scopes, for use in the interpreter AND a potential typechecker.
pub struct VarScope {
    /// this is only zero for the global scope itself
    pub global: Option<Rc<RefCell<Self>>>,
    /// any non-global scope must have a parent, this can also be the global scope
    parent: Option<Rc<RefCell<Self>>>,
    /// the values consist of a behavior specifier (and usually an encapsulated value),
    /// combined with an optional type hint.
    values: HashMap<String, (ScopeReference, Option<StaticValue>)>,
    /// the scope needs to be informed of the `#!strict assign` setting in the interpreter
    pub strict_assign: bool,
}

impl VarScope {
    pub fn find_global_scope(scope: Rc<RefCell<Self>>) -> Rc<RefCell<Self>> {
        scope.as_ref().borrow().global.clone().unwrap_or(scope.clone())
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
    pub fn new_local(parent: Rc<RefCell<Self>>, strict_assign: bool) -> Self {
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
    pub fn get(&self, id: &str) -> StaticValue {
        return match self.values.get(id) {
            Some((ScopeReference::LocalValue(o), _)) => o.clone(),
            // Propagate is never actually used, but a key not being present implicitly behaves the same way.
            Some((ScopeReference::Propagate, _)) | None => match &self.parent {
                Some(o) => o.as_ref().borrow().get(id),
                None => StaticValue::None,
            },
            Some((ScopeReference::Global, _)) => match &self.global {
                Some(o) => o.as_ref().borrow().get(id),
                None => StaticValue::None,
            },
            Some((ScopeReference::Local, _)) => StaticValue::None,
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
                Some(o) => o.as_ref().borrow().is_declared(id),
                None => false,
            },
            Some((ScopeReference::Propagate, _)) | None => match &self.parent {
                Some(o) => o.as_ref().borrow().is_declared(id),
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
                Some(o) => o.as_ref().borrow().has_value(id),
                None => false,
            },
            Some((ScopeReference::Propagate, _)) | None => match &self.parent {
                Some(o) => o.as_ref().borrow().has_value(id),
                None => false,
            },
        };
    }
    /**
    Get the type hint for a variable, if it has been declared with one.
    TODO: can global variables be declared with a type hint that differs from their global definition?
    It could make sense if global variables are always typeless, or if one can't rely on their type hints well...
     */
    pub fn get_type_hint(&self, id: &str) -> Option<StaticValue> {
        return match self.values.get(id) {
            Some((ScopeReference::Local, o)) | Some((ScopeReference::LocalValue(_), o)) => {
                o.clone()
            }
            Some((ScopeReference::Global, o)) => o.clone().or(match &self.global {
                Some(o) => o.as_ref().borrow().get_type_hint(id),
                None => None,
            }),
            Some((ScopeReference::Propagate, _)) | None => match &self.parent {
                Some(o) => o.as_ref().borrow().get_type_hint(id),
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
    pub fn set(&mut self, id: &str, val: StaticValue) -> bool {
        if self.is_declared(id) {
            match self.values.get(id) {
                Some((ScopeReference::LocalValue(_),_)) | Some((ScopeReference::Local,_)) => {
                    let o = self.values.remove(id).unwrap();
                    self.values.insert(id.to_owned(), (ScopeReference::LocalValue(val), o.1));
                },
                Some((ScopeReference::Global,_)) => match self.global {
                    Some(ref g) => return g.as_ref().borrow_mut().set(id, val),
                    None => panic!("the global scope isn't supposed to defer to Global")
                },
                Some((ScopeReference::Propagate,_)) | None => return self.parent.as_deref()
                    .expect("if a propagating variable is declared, then the parent must hold that declaration,
                    and in particular the parent must exist.").borrow_mut().set(id, val),
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
    pub fn declare(&mut self, id: &str, value: ScopeReference, type_hint: Option<StaticValue>) {
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
        global_scope: Rc<RefCell<VarScope>>,
        local_scope: Option<VarScope>,
    ) -> (VarScope, Result<StaticValue, StaticValue>) {
        let mut local_scope =
            local_scope.unwrap_or_else(|| VarScope::new_local(global_scope.clone(), false));
        let mut value_stack = Vec::<StaticValue>::new();

        // the obtained result is pre-stored here
        let result = try {
            for inst in instructions {
                match inst {
                    Statement::LoadStatic(val) => value_stack.push(val.clone()),
                    Statement::BinOperator(op) => {
                        let b = value_stack.pop();
                        let a = value_stack.pop();
                        if let None = a {
                            Err(StaticValue::StringLiteral(
                                "not enough arguments for binary operator".to_owned(),
                            ))?
                        }
                        let (a, b) = (a.unwrap(), b.unwrap());
                        value_stack.push(a.operator(b, op)?);
                    }
                    Statement::LoadScope(id) => value_stack.push(local_scope.get(id)),
                    Statement::LoadGlobal(id) => {
                        value_stack.push(global_scope.as_ref().borrow().get(id));
                    }
                    Statement::StoreScope(id) => {
                        local_scope.set(
                            id,
                            value_stack.pop().ok_or(StaticValue::StringLiteral(
                                "no value to assign".to_owned(),
                            ))?,
                        );
                    }
                    Statement::StoreGlobal(id) => {
                        global_scope.as_ref().borrow_mut().set(
                            id,
                            value_stack.pop().ok_or(StaticValue::StringLiteral(
                                "no value to assign".to_owned(),
                            ))?,
                        );
                    }
                    Statement::Dot(id) => {
                        let a = value_stack.pop();
                        if let None = a {
                            Err(StaticValue::StringLiteral(
                                "no value to index into".to_owned(),
                            ))?
                        }
                        value_stack.push(a.unwrap().dot(id)?)
                    }
                    Statement::Index => todo!(),
                    Statement::Call => todo!(),
                }
            }
            if value_stack.len() > 1 {
                Err(StaticValue::StringLiteral(
                    "too many return values".to_owned(),
                ))?
            } else {
                value_stack.pop().unwrap_or(StaticValue::None)
            }
        };
        // we return the desired result, along with the local scope at that point in time.
        (local_scope, result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, rc::Rc, vec};

    static STRICT_ASSIGN: bool = false;

    #[test]
    fn create_scopes() {
        let global_scope = Rc::new(RefCell::new(VarScope::new_global(STRICT_ASSIGN)));
        let scope1 = Rc::new(RefCell::new(VarScope::new_local(
            global_scope.clone(),
            STRICT_ASSIGN,
        )));
        let scope2 = Rc::new(RefCell::new(VarScope::new_local(
            scope1.clone(),
            STRICT_ASSIGN,
        )));

        assert!(!scope2.as_ref().borrow().is_declared("test"));
        scope1.as_ref().borrow_mut().declare(
            "test",
            ScopeReference::LocalValue(StaticValue::None),
            None,
        );
        assert!(scope2.as_ref().borrow().is_declared("test"));
        assert_eq!(scope2.as_ref().borrow().get("test"), StaticValue::None);
        assert!(!global_scope.as_ref().borrow().is_declared("test"));
    }

    #[test]
    fn interpreter() {
        let global_scope = Rc::new(RefCell::new(VarScope::new_global(STRICT_ASSIGN)));
        let local_scope = VarScope::new_local(global_scope.clone(), STRICT_ASSIGN);

        let instructions = vec![
            Statement::LoadStatic(StaticValue::Int(42)),
            Statement::StoreGlobal("test".to_owned()),
            Statement::LoadScope("test".to_owned()),
        ];
        let (local_scope, res) = StackMachine::exec(&instructions, global_scope, Some(local_scope));

        assert!(local_scope.is_declared("test"));
        assert_eq!(local_scope.get("test"), StaticValue::Int(42));
        assert_eq!(res, Ok(StaticValue::Int(42)));
    }
}
