# The Type System

In its simplest form, an object holds named fields referring to data. These fields may be simply stored values, read- or write-only, or dummies containing complex getset logic. Fields can be declared at any time.

When defining a type, we describe what properties any object of that type will have. In JavaScript this is done via the _Prototype_: any time a function is called, that doesn't exist on the object instance, it is looked up in the prototype and used from there if found. This lookup allows adding functionality to all objects of a specific type at a later point.

This is fine for immutable values like functions, and potentially for values shared among all objects of the type. However a type would usually also define a set of fields that are specific to each instance; the prototype system doesn't work well for that.

The type should expose static methods for object creation, which initialize those fields correctly. Only value fields that are expected to be instance specific should be placed in the objdict; everything else should go into the protodict.