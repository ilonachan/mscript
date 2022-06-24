use std::{
    collections::HashMap,
    mem,
    sync::{Arc, RwLock},
};

use super::types::{string::MshString, MshReference};

/**
Representing a field which can be read and assigned a value. These fields
are primarily used for variables, but also prominently by user-defined types,
especially in the `$dot` and `$index` method.

Instead of immediately fetching the value of the operation then and there, an `AnnotatedField`
can be used as a placeholder for *lazy evaluation*. This allows builtin support for
object member variable assignments, complete with logic guarding them. It's definitely gonna be useful
for destructuring assignments, if I decide to go down that route.
 */
pub trait AnnotatedField {
    fn get_name(&self) -> Option<String>;
    fn get_docstring(&self) -> Option<String>;
    fn can_read(&self) -> bool;
    fn can_write(&self) -> bool;
    fn get_value(&self) -> Result<Option<MshReference>, MshReference>;
    fn set_value(
        &mut self,
        value: Option<MshReference>,
    ) -> Result<Option<MshReference>, MshReference>;
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
    value: Option<MshReference>,
    readonly: bool,
    docstring: Option<String>,
}
impl VariableField {
    pub fn new(
        name: Option<String>,
        value: Option<MshReference>,
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

    fn can_read(&self) -> bool {
        true
    }
    fn can_write(&self) -> bool {
        !self.readonly
    }

    fn get_value(&self) -> Result<Option<MshReference>, MshReference> {
        Ok(self.value.clone())
    }
    fn set_value(
        &mut self,
        value: Option<MshReference>,
    ) -> Result<Option<MshReference>, MshReference> {
        // if the field is readonly, we'll allow setting it for the first time and never again after that.
        if self.readonly && self.value.is_some() {
            return Err(MshString::from("field is readonly").into());
        }
        Ok(mem::replace(&mut self.value, value))
    }
}
impl VariableField {
    pub fn set_docstring(&mut self, docstr: Option<impl Into<String>>) {
        self.docstring = docstr.map(|o| o.into());
    }
}

/** Describing the different ways a variable can be declared */
pub enum VarScopeRefType {
    /// The variable is locally declared. The field itself is then present, although a value need not yet be assigned.
    LocalValue(Arc<RwLock<dyn AnnotatedField>>),
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
    pub fn get(&self, id: &str) -> Option<Arc<RwLock<dyn AnnotatedField>>> {
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
    pub fn get_or_declare(&mut self, id: &str) -> Result<Arc<RwLock<dyn AnnotatedField>>, ()> {
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
                VarScopeRefType::LocalValue(Arc::new(RwLock::new(VariableField::new(
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
