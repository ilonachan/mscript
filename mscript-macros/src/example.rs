pub trait MInt: MNumber + MObject {
    fn get_value(&self) -> isize;
}
pub type MIntRef = Arc<RwLock<dyn MInt>>;

pub struct MIntImpl {
    /// for object functionality.
    mobject: MObjectImpl,
    value: isize,
}
/// simply delegate the object functionality ("composition over inheritance")
impl MObject for MIntImpl {
    delegate! {
      to self.mobject {
        ... // I hope I can just list off all the methods that MObject provides and make that happen
      }
    }
}
impl MNumber for MIntImpl {}
impl MInt for MIntImpl {
    fn get_value(&self) -> isize {
        self.value
    }
}

// ###############################################
// this type definition...
// ###############################################

msh_type! {
    // typeimpl = MIntTypeImpl; // TODO: this should allow me to specify a custom type implementation for use with this.
    name = "int";
    objtrait = MInt;
    supertypes = ["number"];
    impl type {
        /// extremely cursed syntax, but easily readable
        fn "$call"(self, *args, **kwargs, $scope) -> MFuncResult {
            Ok(MIntImpl{value: 42}.wrap())
        }

        /// register a field `my_string` and provide logic that returns its value when accessed.
        /// this docstring will be used for the field. There can only be one `get` for the same name.
        ///
        /// TODO: this functionality makes the most sense if there are hidden attributes for the type
        /// which can only be accessed this way; for that to be relevant, `MIntType` has to be its own trait.
        get my_string(self) -> MFuncResult {
            Ok(MString::from("hello").wrap())
        }
        /// provide logic to set the value of an already registered field. If this is never specified, the field is readonly.
        ///
        /// there can be at most one `set` for the same name. Because write-only fields aren't a thing, any name that has a `set`
        /// needs a `get` or `fn` for the same name.
        ///
        /// This docstring will be ignored.
        set my_string(self, new_val) -> MFuncResult {
            Err(MString::from("there could be code here to make this work").wrap())
        }

        /// `fn` here is a shorthand for creating a field containing a function (a closure containing this function block)
        ///
        /// function fields can be made mutable with `set` as well, but that sounds really dumb so why would you.
        /// You can't have both a `get` and an `fn` for the same name, and no more than one `fn`.
        ///
        /// Like `get` this docstring will be used for the field.
        fn from_str(self, other=MNone::refer(), *myargs, **mykwargs) -> MFuncResult {
            Ok(MNone::refer())
        }
    }
    impl obj {
        fn $add(self, other) -> MFuncResult {
            Ok(other.clone()) //just a placeholder
        }
    }
}

// ###############################################
// ,,,becomes:
// ###############################################

fn create_int_type() -> MTypeRef {
    let mtype = MTypeImpl {
        name: "int".into(),
        objtype: Some(types::builtins::get_type("type")),
        supertypes: vec![types::builtins::get_type("number")],
        proto_dict: HashMap::new(),
        inst_dict: HashMap::new(),
    };

    // initialize the type's own members

    mtype.write().unwrap().inst_dict.insert(
        "$call",
        ObjField::new(
            "$call",
            Some("extremely cursed syntax, but easily readable"),
            // TODO: I want some way to specify the exact type of object passed to a builtin function definition
            // (but maybe that's not a sensible idea? after all the function might be passed around and used on different objects...)
            Some(|| {
                MFunction::Builtin(|self: MObjectRef, args, kwargs, scope| {
                    // because I highly doubt that `self` is a legal parameter name, I replace all occurences in the user code with `__self`.
                    // but it looks like that may not be necessary!
                    let self = try_cast::<MTypeImplRef>(self)?; // idk how to make `try_cast` yet, probably something with typeids
                    Ok(MIntImpl { value: 42 }.wrap())
                })
                .wrap()
            }),
            None,
        ),
    );

    let (get, set) = (mtype.clone(), mtype.clone());
    mtype.write().unwrap().inst_dict.insert(
        "my_string",
        ObjField::new(
            "my_string",
            Some("register a field `my_string`..."), // ellipsis won't actually be part of the generated code
            Some(|| {
                let __self = get; // the ref to self is safely owned
                Ok(MString::from("hello").wrap())
            }),
            Some(|new_val| {
                let __self = set;
                Err(MString::from("there could be code here to make this work").wrap())
            }),
        ),
    );

    mtype.write().unwrap().inst_dict.insert(
        "from_str",
        ObjField::new(
            "from_str",
            Some("`fn` here is a shorthand..."),
            Some(|| {
                MFunction::Builtin(|__self: MObjectRef, myargs, mykwargs, _| {
                    // the names here are decided by the user
                    // if the user has specified positional arguments, we first extract those here.
                    // that's a convenience so the user needn't worry about the rest.
                    // default values are also possible (if I can make that work)
                    let (other, myargs, mykwargs) =
                        extract_positional("other", myargs, mykwargs, Some(|| MNone::refer()));
                    Ok(MNone::refer())
                })
                .wrap()
            }),
            None,
        ),
    );

    // initialize the prototype dictionary, i.e. the default members for instances of the type

    mtype.write().unwrap().proto_dict.insert(
        "$add",
        ObjField::new(
            "$add",
            None,
            Some(|| {
                MFunction::Builtin(|__self: MObjectRef, args, kwargs, scope| {
                    let __self = try_cast::<MIntRef>(__self)?; // this comes from `objtrait`!
                    let (other, args, kwargs) = extract_positional("other", args, kwargs);
                    Ok(other.clone()) //just a placeholder
                })
                .wrap()
            }),
            None,
        ),
    );

    mtype.wrap()
}

types::BUILTINS.create_type("int", create_int_type());
