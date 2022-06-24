# mscript

I'm planning to create a fictional terminal environment, and I want to write a scripting language for the user to interface with it.
The parsing/interpreting/compiling(?)/typechecking(?)/docgen(?) functionality will all be written in Rust (my new favorite language ^\_^)

## Design ideas

**Mindex** is a fictional operating system loosely based on \*nix kernels, designed to work natively on the human brain.
In my setting it provides periphery management, memory encryption & anti-corruption mechanisms,
a builtin firewall, multi-user management etc.  
While the OS can be conveniently controlled using thoughts, just like on real linux systems the need to
interface with the command line will frequently arise; this is done through the **Mindex SHell (MSH)**.
The syntax used by this shell is called **MindScript** or just **mscript**.

That's the chuuni setting out of the way, now to actual design considerations.

Design principles for general interaction with the shell and file system include:

- Unlike current shells, where environment variables are limited to strings, MSH is dynamically typed: stored objects will remember the context they can be used in. Functions are also objects in the namespace, and can have additional attributes such as docstrings.
- Commands are functions, and similar to Python (or even TempleOS) they are invoked with a paren syntax to clearly delineate parameters: instead of `cat test.txt`, one uses `cat("test.txt")`
- Paths and Globs have first-class support: if something starts with `/` or `./`, it will automatically be treated as a path object. In particular, the above example could be clearer if written as `cat(./test.txt)`. (obviously paths can still be constructed from strings, and sometimes this will be necessary)
- Files are callable, if they specify an `#!exec` clause (MSH equivalent for the shebang) in their first line (or the shell has been otherwise told how to execute that file type). Much like current shells, `cat` is not a builtin function, but a callable script placed in a location like `/bin/cat`. The shell can detect that this is a binary and execute it.
- Script files usually have the file ending `.m`, compiled libraries have the ending `.mc`, executables may have the ending `.mx`. All of these are theoretically optional. File paths can't usually omit the file ending, but they can when they are called and there is no ambiguity.

Some concerns regarding the type system:

- primitives include ints, floats, bools, strings.
- builtin compounds include lists, dicts, tuples(?)
- other builtin types are paths, functions, `none`, and types themselves
- types can be dynamically defined and instantiated as well (the exact mechanisms for this are not yet clear)

Regarding the structure of script/general files:

- The first line in a text file may be an `#!exec` instruction. For mscripts this instruction should read `#!exec msh` (this invokes the `msh` interpreter from the global namespace, including the executable path). All file types must admit this.
- At least in mscripts, other such _static instructions_ can be used throughout the file, e.g. to change interpreter settings (activate `#!strict` checks, increase the `#!log` levels or specify an output file,...) or provide editor metadata (like specifying the preferred `#!tab` width, visual cues like `#!sep`arating lines, custom code `#!fold`ing,...)
- Comments start with `#`. A space after that is not required, but the characters `#!<` cause side effects. Block comments are also possible: `#< ... >#`.
- Docstrings can be statically specified for any variable/function, their syntax is `##` or `##< ... >##` respectively.
- Scripts are executed from top to bottom whenever they are used in any way. There are two ways to "use" a script: by running it directly, or by importing it as a library. Code only to be run in the former case can be placed in a `run {...}` block, code for the latter case belongs in an `export {...}` block.
- Scripts have their own isolated scope when executed, from which values do not automatically leak. A script can however explicitly `export` a variable: these will be collected and can be imported on demand (`export`s in an `export`/`run` block will just be passed outside, but the latter will only have any effect if `#!strict import` is not set, i.e. in the user shell). When run in the shell itself, this demand need not be specified; in script files this kind of uncontrolled namespace pollution can be disabled using `#!strict import`

## Progress

- [ ] Define the language grammar
  - [x] Implement a parser (auto-generated using ANTLR4)
- [ ] Create an Interpreter (stack-based)
  - [ ] Use parser result to compile into a simple to execute intermediate form
  - [ ] Define the type system
- [ ] Create OS components around the interpreter, including file interface
  - [ ] Connect OS components to mscript with external libraries
- [ ] **FINAL GOAL:** Integrate the interpreter environment into the Godot Engine for use in a game.

## Build Instructions
It's a `cargo` project, so the usual simplicities apply. I've even done the work of compiling the Antlr4 version with a Rust target for you! If you do wanna build that yourself, you can look at [rrevenantt/antlr4](https://github.com/rrevenantt/antlr4/tree/rust-target) for some pointers; just make sure to change the submodule reference to the latest commit in branch `v0.3` of [rrevenantt/antlr4rust](https://github.com/rrevenantt/antlr4rust/tree/v0.3), because that's the version this project requires.

Once you've placed the compiled tool jar in the project root as `antlr4.jar` you're good to go! `cargo` will automatically recompile the lexer/parser if you change the grammar. (And if the Rust target ever gets merged into Antlr4 proper, you can adapt `build.rs` to call the official version on your `$PATH` instead)

## License
Let's just say MIT. I really don't care much about what you do with this code, in all likelihood it's too specific to be useful for any broad application anyway. If I do manage to write a language that is cool and helpful and nice to use and works, please feel free to use it as a reference for your own work.