use std::{
    borrow::Borrow,
    cell::RefCell,
    collections::HashMap,
    env::var,
    fmt::{Debug, Display},
    hash::Hash,
    mem,
    rc::Rc,
    sync::{Arc, RwLock},
};

#[derive(Clone)]
pub enum MshValue {
    /// Nothing. Returned by functions that don't return anything,
    /// the value of a variable that hasn't been assigned a value,
    /// the result of safe-ish access operations to places that don't exist,...
    None,
    /// Integer numbers. Currently stored as `isize`, which is unreliable
    /// and I want to replace it with something like a `BigInt` at some point
    /// (if that already exists in Rust, which lbr there's no world where it doesn't)
    Int(isize),
    /// Floats. Similarly, maybe there's a "theoretically unlimited precision type"
    /// that I can use for this.
    Float(f64),
    /// Booleans, very simple.
    Bool(bool),
    /// Strings are immutable, and so easily identified that I'll just leave them here.
    Str(String),
    Obj(Rc<RefCell<Box<dyn MshObject>>>),
}
impl Debug for MshValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self.str_debug() {
                Ok(o) => o.unwrap_str(),
                Err(e) => format!("<error in $dbgstr: {}>", e),
            }
        )
    }
}
impl Display for MshValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self.str_nice() {
                Ok(o) => o.unwrap_str(),
                Err(e) => format!("<error in $str: {}>", e),
            }
        )
    }
}
impl MshValue {
    fn objtype(&self) -> MshValue {
        match self {
            MshValue::None => todo!(),
            MshValue::Int(_) => todo!(),
            MshValue::Float(_) => todo!(),
            MshValue::Bool(_) => todo!(),
            MshValue::Str(_) => todo!(),
            MshValue::Obj(o) => o.as_ref().borrow().objtype(),
        }
    }
    fn str_nice(&self) -> Result<MshValue, MshValue> {
        match self {
            MshValue::Str(s) => Ok(Self::Str(s.to_owned())),
            MshValue::Obj(o) => o.as_ref().borrow().str_nice(),
            _ => self.str_debug(),
        }
    }
    fn str_debug(&self) -> Result<MshValue, MshValue> {
        match self {
            MshValue::None => Ok(Self::Str("none".to_owned())),
            MshValue::Int(i) => Ok(Self::Str(i.to_string())),
            MshValue::Float(f) => Ok(Self::Str(f.to_string())),
            MshValue::Bool(b) => Ok(Self::Str(b.to_string())),
            MshValue::Str(s) => Ok(Self::Str(format!("'{}'", s.to_owned()))),
            MshValue::Obj(o) => o.as_ref().borrow().str_debug(),
        }
    }

    /**
    Dot syntax is used to get a field for an object. In Mscript, the dot operator
    can be implemented by the user in the following way:
    * the function is called whenever anyone tries to access the specified member, and returns:
      * Nothing (`none`) if the field doesn't exist
      * A field-like object: one that specifically implements `get`, `set`, and provides
        the `name`,`docstring`,`readonly`,`typehint` info.
      * If the return value isn't field-like, it is automatically wrapped in an anonymous readonly field.
    * For builtins the function may return `VariableField`s, because those are convenient for static instantiation.
    * Mscript objects will have to be put in a wrapper to work.

    */
    fn dot(&self, identifier: String) -> Result<Rc<RefCell<dyn AnnotatedField>>, MshValue> {
        todo!();
        Err(MshValue::Str("not implemented".into()))
    }

    /**

    */
    fn call(
        &self,
        _args: Vec<MshValue>,
        _kwargs: HashMap<String, MshValue>,
        _scope: Rc<RefCell<VarScope>>,
    ) -> Result<MshValue, MshValue> {
        match self {
            MshValue::Obj(o) => o.as_ref().borrow().call(_args, _kwargs, _scope),
            _ => Err(MshValue::Str(format!("cannot call `{:?}`", self.objtype()))),
        }
    }

    /// the string methods are guaranteed to return one, but I can't specify this
    /// without breaking the enum return. So unless I decide to go that route, I can try unwrapping instead.
    fn unwrap_str(&self) -> String {
        match self {
            MshValue::Str(s) => s.to_owned(),
            _ => panic!("Tried to use `unwrap_str` on a non-string value"),
        }
    }
}

pub trait MshObject: Debug {
    fn objtype(&self) -> MshValue;
    fn str_nice(&self) -> Result<MshValue, MshValue>;
    fn str_debug(&self) -> Result<MshValue, MshValue>;
    fn call(
        &self,
        _args: Vec<MshValue>,
        _kwargs: HashMap<String, MshValue>,
        _scope: Rc<RefCell<VarScope>>,
    ) -> Result<MshValue, MshValue>;
}

pub trait AnnotatedField {
    fn get_name(&self) -> Option<String>;
    fn get_docstring(&self) -> Option<String>;
    fn is_readonly(&self) -> bool;
    fn get_value(&self) -> Result<Option<MshValue>, MshValue>;
    fn set_value(&mut self, value: Option<MshValue>) -> Result<Option<MshValue>, MshValue>;
}

/**
An object representing a (possibly unassigned) field or variable. It can obviously serve as a marker for
variables that were declared but not yet assigned, but it also keeps track of some important state:
* whether the variable is readonly (eg a `const` variable or a builtin field)
* the docstring of the variable (not stored in the object so it persists after reassignment)
* a type hint (not yet implemented)

TODO: Variables for general values NEED to store the docstring in the annotations, but what about functions?
I believe it makes sense to add those docstrings both to the annotations AND the function itself.
*/
pub struct VariableField {
    name: Option<String>,
    value: Option<MshValue>,
    readonly: bool,
    docstring: Option<String>,
}
impl VariableField {
    pub fn new(
        name: Option<String>,
        value: Option<MshValue>,
        readonly: bool,
        docstring: Option<String>,
    ) -> Self {
        VariableField {
            name,
            value,
            readonly,
            docstring,
        }
    }
}
impl AnnotatedField for VariableField {
    fn get_name(&self) -> Option<String> {
        self.name.as_ref().map(|o| o.to_owned())
    }
    fn get_docstring(&self) -> Option<String> {
        self.docstring.as_ref().map(|o| o.to_owned())
    }

    fn is_readonly(&self) -> bool {
        self.readonly
    }

    fn get_value(&self) -> Result<Option<MshValue>, MshValue> {
        Ok(self.value.clone())
    }
    fn set_value(&mut self, value: Option<MshValue>) -> Result<Option<MshValue>, MshValue> {
        // if the field is readonly, we'll allow setting it for the first time and never again after that.
        if self.readonly && self.value.is_some() {
            return Err(MshValue::Str("field is readonly".into()));
        }
        Ok(mem::replace(&mut self.value, value))
    }
}
impl VariableField {
    pub fn set_docstring(&mut self, docstr: Option<impl Into<String>>) {
        self.docstring = docstr.map(|o| o.into());
    }
}

pub enum VarScopeRefType {
    /// The variable is locally declared. The field itself is then present, although a value need not yet be assigned.
    LocalValue(Rc<RefCell<VariableField>>),
    /// Never actually used: just a marker to signify the field being declared in `get_behavior`
    Local,
    /// The variable explicitly defers to the global scope.
    Global,
    /// The variable isn't defined in this scope, so any access implicitly propagates.
    /// (not used internally)
    Propagate,
}
pub struct VarScope {
    global: Option<Rc<RefCell<Self>>>,
    parent: Option<Rc<RefCell<Self>>>,
    variables: HashMap<String, VarScopeRefType>,
    pub strict_assign: bool,
}
impl VarScope {
    pub fn find_global_scope(scope: Rc<RefCell<Self>>) -> Rc<RefCell<Self>> {
        scope
            .as_ref()
            .borrow()
            .global
            .clone()
            .unwrap_or(scope.clone())
    }

    /// Create a new global scope as a root for the scope tree
    pub fn new_global(strict_assign: bool) -> Self {
        VarScope {
            global: None,
            parent: None,
            variables: HashMap::new(),
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
            variables: HashMap::new(),
            strict_assign,
        }
    }

    /**
    Get the field under the specified name in the scope. If the field isn't declared or defers to global,
    the call is propagated upwards in the scope tree.

    This function always returns a value according to the defined algorithm,
    and `None` even if a variable isn't declared. if raw information about
    the scope is needed, use `get_behavior`, `is_declared` or `has_value` instead.
    */
    pub fn get(&self, id: &str) -> Option<Rc<RefCell<VariableField>>> {
        return match self.variables.get(id) {
            Some(VarScopeRefType::LocalValue(o)) => Some(o.clone()),
            // Propagate is never actually used, but a key not being present implicitly behaves the same way.
            Some(VarScopeRefType::Propagate) | None => self.parent.as_ref().and_then(|o| o.as_ref().borrow().get(id).clone()),
            Some(VarScopeRefType::Global)           => self.global.as_ref().and_then(|o| o.as_ref().borrow().get(id).clone()),
            Some(VarScopeRefType::Local) => {
                panic!("this variant shouldn't actually be used internally")
            }
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
    pub fn get_behavior(&self, id: &str) -> VarScopeRefType {
        return match self.variables.get(id) {
            Some(VarScopeRefType::LocalValue(_)) | Some(VarScopeRefType::Local) => {
                VarScopeRefType::Local
            }
            // Propagate is never actually used, but a key not being present implicitly behaves the same way.
            Some(VarScopeRefType::Propagate) | None => VarScopeRefType::Propagate,
            Some(VarScopeRefType::Global) => VarScopeRefType::Global,
        };
    }

    /**
    Set the value of a variable in scope, deferring to the parent/global scope if necessary.

    If a variable isn't declared yet, it will be declared locally with the new value.
    However this only happens if `#!strict assign` isn't set; if it is the function simply returns `false`.
    It returns `true` iff the value was correctly assigned, which is true in every other case.
     */
    pub fn get_or_declare(&mut self, id: &str) -> Result<Rc<RefCell<VariableField>>, ()> {
        if self.get(id).is_none() && !self.strict_assign {
            self.declare(id, VarScopeRefType::Local);
        }
        self.get(id).ok_or(())
    }

    /**
    Declare a new variable in the scope; can also redeclare existing variables,
    and undeclare variables if `Propagate` is specified. An optional type hint can be provided.
     */
    pub fn declare(&mut self, id: &str, variant: VarScopeRefType) {
        match variant {
            VarScopeRefType::Local => self.variables.insert(
                id.to_owned(),
                VarScopeRefType::LocalValue(Rc::new(RefCell::new(VariableField::new(
                    Some(id.to_owned()),
                    None,
                    false,
                    None,
                )))),
            ),
            VarScopeRefType::Propagate => self.variables.remove(id),
            _ => self.variables.insert(id.to_owned(), variant),
        };
    }
}
