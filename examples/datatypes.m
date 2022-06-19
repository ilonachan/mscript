#!exec msh
# we can expect important functions to be in scope, this includes the system-wide interpreter.

run {
  # there are only a handful of builtin types:
  # numbers are self-explanatory
  local a: int = +1234
  local b: float = 123.456e-7
  # strings are immutable
  local c: str = "abc"
  # boolean logic, also very obvious
  local d: bool = true || false

  # functions are also first-class objects: this will just put an object of type `func` into the variable `f`.
  func f(a1: str, a2: int = 0) -> bool {
    return true
  }
  # and this function can later be redefined:
  f = func () -> str {return "completely different"}    # This syntax might give the compiler headaches though...

  # exceedingly clear and more compact: lambda notation. For when a full function logic block would just be boilerplate.
  f = func () => "look how slick it is!"
}

# Now we get to the interesting part: structs. The following creates a new type `MyDataStructure`,
# which can later be instantiated:
struct MyDataStructure {
  export name: str
  # default values can be specified and are calculated right away. The result is assigned to each new object.
  export age: int = 0
  # any property is private by default; export it so others can use it.
  local private_thing: str = "secret"

  # functions are obviously also something that can be fit in a struct.
  # this already makes it feel like OOP!
  # the function will be executed in a closure of the struct as it was created, 
  export func get_secret() -> str {
    return "haha I lied"
  }

  # internally when a struct is instantiated, the code inside the block is run
  print("")
}

# We can add some properties to the `MyDataStructure` type itself, like static methods
type MyDataStructure {

}


# in terms of predefined object types, there are quite a few:

# obviously, lists.
local mylist: list[any] = [1,2,"stuffsies"]
# dicts are also pretty obvious if you ask me.
local mydict: dict[str, int] #< = dict[int] ># = {"hello": 1, "world": 2}
# i'm torn on whether tuples make sense, or if that's too much reinventing python...
local tup: (int,int) = (1,2)
local tup2: (str,) = ("single element tuple (y tho)",)

# it'd be incredible if destructuring assignment was possible, but it's very likely impractical
local (a2,b2) = ("y.y.", "hannya")

# since mscript is a shell scripting language, for use in a file system environment,
# paths and calling files get first-class support. A path object can refer to a single file/directory,
# or to a list of paths as defined by a glob or manual OR'ing of paths.
# operations like appending, traversal, iteration,... run over all options simultaneously
# but of course, if only a first path match is required, that can be indexed as well.
local e: path = /path/to/some/file.m || /path/to/a/folder || ./all/*folders/in\ a/**/.glob


