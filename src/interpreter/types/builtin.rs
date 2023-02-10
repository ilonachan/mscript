use lazy_static::lazy_static;

use std::collections::HashMap;

use super::{MTypeImpl, MTypeRef, none, object};

lazy_static! {
    pub static ref BUILTINS: Builtins = Builtins::singleton();
}

struct Builtins {
    types: HashMap<String, MTypeRef>,
}
impl Builtins {
    fn singleton() -> Self {
        Self;
    }
    pub fn create_type(&mut self, _type: MTypeRef) {
        self.types
            .insert(_type.read().unwrap().name(), _type.clone())
    }
    pub fn get_type(&self, name: &str) -> MTypeRef {
        self.types.get(name).clone()
    }
}

fn create_type_type() -> MTypeRef {
    let type_type = MTypeImpl::new("type", None, vec![BUILTINS.get_type("obj")]).wrap();
    type_type
}

fn init_type_system() {
    BUILTINS.create_type(object::create_object_type());
    BUILTINS.create_type(create_type_type());
    BUILTINS.create_type(none::create_none_type());
}
