#!exec /bin/msh
#!tab 2
#!strict none

#<
the above are static instructions, which are much simpler than the rest of the language,
and can be used similar to shebangs: to easily convey basic information to the shell or an editor.

> !exec is the direct equivalent of unix shebangs. I think it'd be good if syntax like
  #!exec /bin/msh($file, enc='windows') was allowed.
  Then "#!exec /bin/msh" would be a shorthand for #!exec /bin/msh($file)
  (with $file obivously specifying the current file's path)

> !tab can be used to tell an editor how wide a tab should be displayed. Obviously editors aren't forced
  to respect this, but it'd be great to at least get a warning in case of mismatch

> !strict is used to enable additional typechecking/security features. A few unsafe shorthands
  are present and useful in the shell itself, but dangerous in production code; !strict allows
  the programmer to enforce safer default behaviours in the interpreter.

  possible values include: import, assign, dolstr, none (to explicitly disable all, default), all (to explicitly enable all)
  not speficying !strict is equivalent to 'none', multiple toggles can be separated by spaces (#!strict import assign)
>#


# this is how libraries are "imported":
# > if the value "console" is defined in the shell, it is used
# > otherwise os functions are used to load the library
# > if the library doesn't exist or can't be loaded, an error is reported
# > a library loaded this way may be cached by the OS, but its accessibility is limited to this file.

#< 
# ignore all of this, it's an old attempt at conceptualizing imports
# macros may never be a relevant idea, as code transformation isn't really workable in the msh interpreter

!!load console
!!load /lib/console.mc
!!load ./path/to/myconsole.m as console
# lines like this are macros: shorthands for more elaborate syntax.
# I don't yet know how to let the user define these macros himself
# ...or what syntax to even use for them
# ...or if there even IS a use for them, but maybe I'll just add it and hope for the best?
# the expanded version of the !!load macro is shown below:

local console
if not defined(console) {
  console = /bin/load(console)    # yeah some shell files can just be called like this.
                                  # this specific implementation will find "console.m" or "console.mc" in the LIBPATH
                                  # (mc is for compiled libraries, which I may just use to obscure interfaces
                                  # to the containing game engine and code I don't necessarily want/need to show)

  # these are alternate branches based on the input to load, and this makes me think
  # I should just have a builtin function $load instead for unified syntax...
  console = /bin/load(/lib/console.mc)       # if the path to a file was specified. the name will be used for the assigned symbol.
  console = /bin/load(./path/to/myconsole.m) # if you don't want that, specify the import name.

  # why would you not just call /lib/console.mc() you ask? ...I don't know.
  # > Maybe the $call function of a module can have side effects that /bin/load avoids?
  # > Maybe /bin/load prevents console.mc from polluting the namespace, or specifically allows that?
  # > either way, it leads to the unified syntax above.
}
>#
import console=stdout

#< random tech demo: paths
in shells it's usual to write out file paths, and mscript directly supports this. A file path is a syntactically correct object,
which is then turned into an internal object with helper functions and such.

```msh
# valid path syntax includes:
/absolute/path/to/file.m
./relative/path/to/file.mc
$HOME/variables/are/supported.txt
/omit/file/ending/if/the/name/is/unique
/select/a/directory/

# i really want to make these support glob syntax as well:
/path/to/{this;that}/file
/path/to/many/files/*
./big/folder/substruct/**/*.txt
$PATH/contains/multiple/entries

# glob paths should be usable as lists of regular paths, in some canonical order.
# the next line would match the highest-priority command for 
```

Path objects are callable: for mscript files this runs their contents in a shell.
For other files the #!exec static instruction can specify an interpreter that can be called as a proxy. Importing works the same way.
If no #!exec instruction is present, the interpreter can be manually invoked.

example: `/path/to/my/script("world")`, `local data = /bin/json(./my/file.json).load()`
>#

# static instructions can be used for many fun editor tricks, such as drawing a separator line between parts of the code
# (again the editor need not respect this, but it'd be cool if it did.)
#!sep

## documentation can be written like this
## it should also be foldable in the editor
## 
## !param name the name of the user
## !return nothing
func hello(name: str) -> none {

  # normal comments are started with a # character
  #< should I include block comments?
  If they exist this is what they should be like.
  Very simple to understand, right? >#
  
  console.println('hello '+name+'!')  # dot syntax, string concatenation
  console.println(f'hello $name, ${name} or $(capitalize(name))')
}

##<
Extremely long docstrings can also be specified in a block-y way.

I think I may actually prefer this. With proper highlighting it'd look pretty sweet.

!param str the string to capitalize
!return a capitalized version of `str`
>##
func capitalize(s: str) -> str {

  return s.upper();  # built-in types are objects with a prototype
                     # the semicolon at the end is optional: unless more tokens in brackets etc
                     # are needed, a line break terminates a command. If you want to explicitly
                     # break a command in the middle of a line, use \ like in Python or bash.
}

# constants are nice, but I don't want to allow polluting the global namespace with things the user can't get rid of;
# so they are always confined to the local file. How all of this will work (or if it's even needed) is questionable.
const v1: int = 12
# normal local variables. These lose their meaning after the scope ends.
local v2: int = 24
v2+=v1;
# global binds the variable to the shell's global scope.
global v3 = v2;

# how about an export keyword allowing a variable to be hoisted to the outer scope?
export v4 = 'hello caller';
# a caller could either import everything into a struct using
#import hello_world()
#import hello_2 = hello_world()
# or specify which variables to hoist:
#import * from ./path/to/hello_world.m()
#import v4, my_name = v5 from hello_world()
# these variables are then local, mutable, and can be reexported if needed.

# actually I really like this! It just makes things so much more consistent!
# maybe this should just be my import concept then.

## it is possible to specify custom code folding via static instructions.
## these preceding lines should also be picked up, and associated with the block.
## they should replace the rest of the content when the block is folded together.
## if you want a block to stop showing its shorthand, use "#!bfold neura nocomment".
#!fold neura

import neura

local stim: neura.Stimulus = neura.cortex.visual.last_stimulus()
export stim_ref = stim  # normally assignment is by-reference for objects
export stim_cp <= stim  # assignment can be made by-value explicitly, this invokes stim.$clone
                        # $identifier should be reserved for internal functionality,
                        # eg $env might be automatically populated for a file, or 
                        # .$eq on an object defines the magic method to check equality, etc

#!endfold neura

