# Tua

## It's Lua with Types

**This is just a language that I'm thinking of making. It would be interpreted (maybe jit compiled in the future), strongly typed, and reference counted.**

**The main goals of this language are to look simple and easy to understand, to be easy to use, and to be a good choice for building large programs.**

<br>
<br>
<br>

## Built-in Types:

### Primitive Types (copied)

- **int** (same as isize in Rust)
- **uint** (same as usize in Rust)
- **int_8**
- **int_16**
- **int_32**
- **int_64**
- **uint_8**
- **uint_16**
- **uint_32**
- **uint_64**
- **float**
- **float_64**
- **bool**

### Complex Types (reference counted)

- String
- Array <type>
- HashMap <key_type, value_type>

### Aliases

- type char = uint_32

### STD

- Optional <type> (implemented in compiler)
- MaybeError <type, error_type>

<br>
<br>
<br>

## Type System

**This is basically Rust's type system but with new names. There are some differences, though (not explained here)**

### Objects

**This is Tua's name for structs. Example:**

```
object Person (
	name: String,
	job: Optional<Job>,
	is_avive: bool,
)

var person = new Person (
	name: "what42",
	job: Job.Programmer,
	is_alive: true,
)
```

### Choices

**This is Tua's name for enums. Example:**

```
choice Optional (
	Filled (any_inner),
	Nothing,
)

var twitter = Nothing
```

### Generics

**Any type name that is "any" or starts with "any_" is a generic type. Example:**

```
choice Optional (
	Filled (any_inner),
	Nothing,
)
```

<br>
<br>
<br>

## Error Handling:

**Instead of Result<T, E>, Tua has MaybeError<any_ok, any_error>. Expample:**

```
#makeDefaultConstructor
object Person (
	name: String,
	age: uint,
	#notInConstructor
	is_alive: bool,
)

// since no second generic arg is given for MaybeError, it is implied to be an anonymous `choice`
// full return type: MaybeError<PersonData, (InvalidInputLength or InvalidUInt)>
function parse_person_data (input: String) returns MaybeError<PersonData>
	var input_parts = input.split('`')
	
	var length = input_parts.length()
	if length != 2 then
		return Error(InvalidInputLength.new(length))
	end
	
	var name = input_parts[0]
	var age = test input_parts[1].to_uint()
	return PersonData.new(name, age)
	
end

#makeDefaultConstructor
object InvalidInputLength (
	length: int,
)
```