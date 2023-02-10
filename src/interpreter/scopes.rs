use std::{
    collections::HashMap,
    mem,
    sync::{Arc, RwLock}, num::NonZeroIsize,
};

use super::types::{MObjectRef, MFuncResult, string::MStringImpl};

pub type MFieldResult = Result<Option<MObjectRef>, MObjectRef>;

/**
Representing a field which can be read and assigned a value. These fields
are primarily used for variables, but also prominently by user-defined types,
especially in the `$dot` and `$index` method.

Instead of immediately fetching the value of the operation then and there, an `AnnotatedField`
can be used as a placeholder for *lazy evaluation*. This allows builtin support for
object member variable assignments, complete with logic guarding them. It's definitely gonna be useful
for destructuring assignments, if I decide to go down that route.
 */
pub trait Field {
    fn name(&self) -> String;
    fn docstring(&self) -> Option<String>;
    fn can_read(&self) -> bool;
    fn can_write(&self) -> bool;
    fn get(&self) -> MFieldResult;
    fn set(&mut self, new_value: Option<MObjectRef>) -> MFieldResult;
    fn del(&mut self) -> MFieldResult;
}
pub type FieldRef = Arc<RwLock<dyn Field>>;

pub struct DynamicField {
    name: String,
    docstring: Option<String>,
    get: Option<dyn Fn() -> MFuncResult>,
    set: Option<dyn Fn(MObjectRef) -> MFuncResult>,
    del: Option<dyn Fn() -> MFuncResult>,
}
impl DynamicField {
    pub fn new(
        name: String,
        docstring: Option<String>,
        get: Option<impl Fn() -> MFuncResult>,
        set: Option<impl Fn(MObjectRef) -> MFuncResult>,
        del: Option<impl Fn() -> MFuncResult>,
    ) {
        DynamicField {
            name,
            docstring,
            get,
            set,
            del
        }
    }
}
impl Field for DynamicField {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn docstring(&self) -> Option<String> {
        self.docstring.clone()
    }
    fn can_read(&self) -> bool {
        self.get.is_some()
    }
    fn can_write(&self) -> bool {
        self.set.is_some()
    }
    fn get(&self) -> MFieldResult {
        match self.get {
            Some(func) => func().map(|v| Some(v)),
            None => None
        }
    }
    fn set(&mut self, new_value: MObjectRef) -> MFieldResult {
        match self.set {
            Some(func) => func(new_value).map(|v| Some(v)),
            None => None
        }
    }
    fn del(&self) -> MFieldResult {
        match self.del {
            Some(func) => func().map(|v| Some(v)),
            None => None
        }
    }
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
pub struct StaticField {
    name: String,
    docstring: Option<String>,
    value: Option<MObjectRef>,
    readonly: bool
}
impl StaticField {
    pub fn new(
        name: Option<String>,
        docstring: Option<String>,
        value: Option<MObjectRef>,
        readonly: bool,
    ) -> Self {
        StaticField {
            name,
            docstring,
            value,
            readonly,
        }
    }
}
impl Field for StaticField {
    fn name(&self) -> String {
        self.name.as_ref().map(|o| o.to_owned())
    }
    fn docstring(&self) -> Option<String> {
        self.docstring.as_ref().map(|o| o.to_owned())
    }

    fn can_read(&self) -> bool {
        true
    }
    fn can_write(&self) -> bool {
        !self.readonly
    }

    fn get(&self) -> MFieldResult {
        Ok(self.value.clone())
    }
    fn set(
        &mut self,
        value: MObjectRef,
    ) -> MFieldResult {
        // if the field is readonly, we'll allow setting it for the first time and never again after that.
        if self.readonly && self.value.is_some() {
            return Err(MStringImpl::from("readonly fields can only be assigned once").wrap());
        }
        Ok(mem::replace(&mut self.value, value))
    }
    fn del(&mut self) {
        if self.readonly {
            return Err(MStringImpl::from("readonly fields cannot be deleted").wrap());
        }
        Ok(mem::replace(&mut self.value, None))
    }
}
// impl StaticField {
//     pub fn set_docstring(&mut self, docstr: Option<impl Into<String>>) {
//         self.docstring = docstr.map(|o| o.into());
//     }
// }

/** Describing the different ways a variable can be declared */
pub enum VarScopeRefType {
    /// The variable is locally declared. The field itself is then present, although a value need not yet be assigned.
    LocalValue(FieldRef),
    /// Never actually used: just a marker to signify the field being declared in `get_behavior`
    Local,
    /// The variable explicitly defers to the global scope.
    Global,
    /// The variable isn't defined in this scope, so any access implicitly propagates.
    /// (not used internally)
    Propagate,
}
/**
Describes scopes in the msh runtime. The VarScope holds definitions of all known variables in fields.
 */
pub struct VarScope {
    global: Option<Arc<RwLock<Self>>>,
    parent: Option<Arc<RwLock<Self>>>,
    variables: HashMap<String, VarScopeRefType>,
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
            variables: HashMap::new(),
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
    pub fn get(&self, id: &str) -> Option<FieldRef> {
        return match self.variables.get(id) {
            Some(VarScopeRefType::LocalValue(o)) => Some(o.clone()),
            // Propagate is never actually used, but a key not being present implicitly behaves the same way.
            Some(VarScopeRefType::Propagate) | None => self
                .parent
                .as_ref()
                .and_then(|o| o.as_ref().read().unwrap().get(id).clone()),
            Some(VarScopeRefType::Global) => self
                .global
                .as_ref()
                .and_then(|o| o.as_ref().read().unwrap().get(id).clone()),
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
    pub fn get_or_declare(&mut self, id: &str) -> Result<FieldRef, ()> {
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
                VarScopeRefType::LocalValue(Arc::new(RwLock::new(StaticField::new(
                    Some(id.to_owned()),
                    None,
                    None,
                    false,
                )))),
            ),
            VarScopeRefType::Propagate => self.variables.remove(id),
            _ => self.variables.insert(id.to_owned(), variant),
        };
    }
}
