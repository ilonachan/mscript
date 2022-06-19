#!exec /bin/msh
#!strict import assign dolstr

#!tab 2

#< !strict import
Within functions, the `export` keyword can be used to hoist a value to the calling scope.
This will always happen when a function is called, possibly without the user's consent.
In the shell this is convenient for the user, while in scripts it can pose security concerns;
to avoid this, a !strict toggle can disable that behavior.

```msh
# collects all the exports into a struct m1 in the calling scope
import m1=module()
# collect only the specified exports and dump them under the specified name
# many different shorthands for element selection/organization are supported.
import el1, flol=el2, n1.flel=el3.subel, n2.=el4, n2.{el5, fsheesh=el5} from module()
# dumps all exports directly into the scope (this is clearly visible in the code)
import * from module()

# if !strict import is unset, this is equivalent to the last option (except FAR less obvious!)
# otherwise it's just a function call without export side effects
module()

# exports are collected from a function call, but functions may also return values themselves.
# this value can be accessed as follows (all other import variants can also be used):
local result = import module.with_sideeffects("hi") 
```
>#
import stdout()

# .m files or even compiled .mc files can just be called as functions in msh.
# But their contents can only be accessed after they've been imported.
# (file endings can be omitted if there's only one file with that name in the directory)
# (it's also possible to call other file types this way,
#   this is handled via an !exec static instruction in the target file)
import ./libb()
import submod=libb.i_export_stuff()

# TODO: objects could also be importable, this is helpful to reorganize the structure of already imported modules
import t1=el3 from {el1:"banishment",el2:"this",el3:"world"}

# TODO: when using `import funccall()`, where are the results stored? what happens with the original function?
# > if funccall is actually a file, its simple name (without ending) is used. Nothing changes.
# > if funccall is not a file, but a reference to a file in the PATH, that simple name is used. The PATH file is henceforth inaccessible. (???)
# > if funccall is a function, it'd be obvious to use its name. However, this would DELETE the function!
#      Is this intentional? should all imports be one-time?
#      should the original function merged into the new dict's $call?
#      what if the user has exported a $call?
# > does it make sense to allow omitting the parens for functions without arguments? when providing a file `import` knows what to do;
#      when providing a path file name or a function, similarly. Does the ability to import objects justify making all of this impossible?
# > PROPOSAL: add a magic method $import, which is called with the arguments to the funccall (if parens are present) or without.
#      For files the default implementation is just $call, which is just the code of the file.
#      For functions this is also just $call.
#      For other objects a function containing `export`s can be defined arbitrarily; if no such method is found an error is raised
#      An imported object can reexport a new $import function, which allows modules to define specific reimport behavior.
#      The default implementation should just return the object itself without side effects, for cached modules.

# TODO: two additional clauses can control how a script reacts depending on how it's invoked:
export {
  # this code is only run when the script is imported.
  # It could be a good place for exported variables, but it's probably useful to still keep them outside...
  # so this might end up being useless, but it's also no additional work to implement.
}
run {
  # this code is only run when the script is called.
  # 
}

#< A common pattern I expect to see is
```
func main(*args, **kwargs) {
  # do logic
}
export {
  export $call = main
}
run {
  main(*args, **kwargs)
}
```
this way the script's functionality is executable by file calling,
but at the same time an import doesn't yet trigger any side effects,
allowing a function to be stored in cache by importing it.
>#


#!sep

# constants are pretty easy to understand:
# > variable assigned more than once => error
# > variable used before initialization => error
# > TODO: should it be allowed to separate declaration and assignment, or is that unnecessary? (it might help scope control)
const HALF_THE_ANSWER = 21

export func a() -> str {
  return "hello"
}

# this function would not be exported either way; it is only accessible within the scope of this script.
func b(name: str) -> none {
  stdout.println(name)
}

export func c(name: str) -> int {
  b(name)
  return HALF_THE_ANSWER
}

#!sep

#< !strict assign (TODO)
the `local` keyword initializes a variable. It's a concept present in many languages,
but not in Python or bash. In mscript it is often optional, but I believe it to be
valuable for controlling the scope of a variable:

```msh
# a is bound to the outer scope. local is not optional here.
local a
# and rather than creating a new binding, the outer variable is assigned.
{ a = 20 }

# local could be optional here
local b = 12
# by specifying local in the subscope, we force an inner variable to be created and shadow the outer one.
{ local b = 38 }
# b has not changed here.
```

when assigning a value to a new variable that didn't exist in an outer scope (this includes globals!),
it's not necessary to specify `local`. It could however make the code cleaner to require this.
!strict assign forces the user to always initialize a variable explicitly.
>#

# mutable variables are even simpler than constants:
# > can be assigned to as many times as needed
# > used before assignment => return a none-like value (TODO: make it clear that the variable doesn't exist)
# var undeclared but assigned => implicitly defined as mutable (unless !strict assign)
# var undeclared but used => return none-like value, like in the uninitialized case
local answer = 2*HALF_THE_ANSWER
export const THE_ANSWER = answer

# mutable variables can also be exported. `export local` has the same effect,
# but it's paradoxical and unnecessarily verbose so 'local' can be omitted.
export FUNNY_NUMBER = 69


# global variables are managed by the shell and accessible to everyone everywhere.
# if a variable is used/assigned without being declared locally, but it exists
# globally, the global variable will be used. This can be confusing,
# so before their first use global variables should be explicitly marked as such.
global PATH;
# it's allowed to have this declaration coupled with an assignment.
global PATH = $'$PATH;/home/2748/bin'

# it is also legal to omit the global declaration, however this makes the scope
# of PATH in this file extremely context dependent:
# > if PATH existed in the file through `local path`, this is easy
# > if PATH didn't exist locally, but it did globally, this is equivalent to `global PATH = ....`
# > if PATH exists neither locally nor globally, it will be declared as `local path`(sic!)
# So any later uses of PATH may not know what scope they're assigning to.
PATH = $'$PATH;/home/2748/bin'

# the use of a global variable is also dependent on previous declarations:
# > if PATH was declared locally, that value will be used (even if it's none-like)
# > if PATH was declared globally, that value will be used.
# > if PATH wasn't declared anywhere, the value is none-like
# a use doesn't implicitly declare a variable, so the same situation can repeat later.
stdout.println(PATH)
# if one really really wants to access a global variable despite a local variable of the same name existing,
# the following statements are possible. As long as `local PATH` exists,
# these won't redeclare PATH to be global and only apply to single statements.
stdout.println(global PATH)
global PATH = $'$PATH;/home/2748/bin'

# this can be changed by undeclaring a local variable. If `local PATH` was not defined, this will give a warning and do nothing.
unloc PATH
# it's an expression because why not, so the following works to move a local variable to the global scope:
global PATH = unloc PATH
# this notation is flexible and should be kept, but if a variable should be moved
# to the global scope under the same name the following shorthand would be good:
global PATH