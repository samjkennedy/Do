<div align="center">
  <picture>
        <img src="logo.svg" width="20%" alt="The Do Programming Language">
  </picture> 
</div>

# The Do Programming Language

Do is a stack-based, strongly typed, functional programming language designed around composition, inference, and lists.
The
concatenation of two programs is their composition.

It draws inspiration from Forth, Porth, Factor and Joy, combining postfix syntax with Hindley-Milner-esque
type inference and first-class functions.

## Design Goals

- Understandable syntax, arbitrary symbols are avoided in favour of intuitive keywords where appropriate.
- First-class functions: Lambdas and Higher Order Functions are core.
- Static type checking with type inference.
- Interactive development via a built-in REPL.

## Getting Started

Do is still heavily in development, so be warned that certain features are either not yet implemented for all targets or
are just straight broken.

The REPL and the interpreter are the most stable ways of using Do for now.

Build Do from scratch:

```
$ cargo build --release
```

Enter REPL mode:

```
$ do
(≡) 4 5 + print
9
(≡)
```

For a given .do file:

```
$ cat square.do
fn square (dup *)

[1 2 3 4 5]
    (square) map
    print
```

Interpret a Do file with the -i flag before the file name (Recommended):

```
$ do -i square.do
[1 4 9 16 25]
```

Compile a Do file to .exe (Highly unstable, requires fasm installed, currently only supports 64-bit windows):

```
$ do square.do
Compiled to square.exe
$ square
[1 4 9 16 25]
```

Also supports `-r` to immediately run the compiled file

```
do -r square.do
[1 4 9 16 25]
```

## Core Operations

Each operator consumes a zero or more values from the stack and produces zero or more new values. Here are all the
operators in Do so far:

### Stack Manipulation

| Operation | Signature      | Description             |
|-----------|----------------|-------------------------|
| dup       | a -> a a       | Duplicate top of stack  |
| swap      | a b -> b a     | Swap top two elements   |
| rot       | a b c -> b c a | Rotate top three        |
| pop       | a ->           | Remove top item         |
| over      | a b -> a b a   | Copy second item to top |
| .         | a -> a         | Identity operator       |

### Arithmetic

| Operation | Signature      | Description      |
|-----------|----------------|------------------|
| +         | int int -> int | Addition         |
| -         | int int -> int | Subtraction      |
| *         | int int -> int | Multiplication   |
| /         | int int -> int | Integer division |
| %         | int int -> int | Modulo           |

### Comparison

| Operation | Signature       | Description           |
|-----------|-----------------|-----------------------|
| =         | a a -> bool     | Equality              |
| <         | int int -> bool | Less than             |
| \>        | int int -> bool | Greater than          |
| <=        | int int -> bool | Less than or equal    |
| \>=       | int int -> bool | Greater than or equal |
| !         | bool -> bool    | Boolean negation      |

### Higher-Order Functions

| Operation | Signature                | Description                     |
|-----------|--------------------------|---------------------------------|
| map       | [a] fn(a -> b) -> [b]    | Map function over list          |
| filter    | [a] fn(a -> bool) -> [a] | Keep items that match predicate |
| fold      | [a] fn(a b -> b) b -> b  | Left fold over list             |
| foreach   | [a] fn(a -> ) ->         | Apply function to each element  |

### List Operations

| Operation | Signature      | Description                                |
|-----------|----------------|--------------------------------------------|
| len       | [a] -> int     | Length of a list                           |
| concat    | [a] [a] -> [a] | Concatenate two lists                      |
| head      | [a] -> a       | Return the first element of a list         |
| tail      | [a] -> [a]     | Return all but the first element of a list |

### Misc

| Operation | Signature | Description                        |
|-----------|-----------|------------------------------------|
| print     | a ->      | Print top of stack                 |
| ???       | --        | Debug prints the current typestack |

### Bindings

Stack values can be bound to identifiers with the `let` keyword:

```
4 5 let a b {
    a print     // prints 4
    b print     // prints 5
    a b + print // prints 9
}
```

### Control Flow

Different branches can be executed with `if/else`:

```
10
true if {
    10 +
} else {
    5 +
}
print //prints 20
```

## Example Programs

### Squares of a list

```
[1 2 3 4 5]
    (dup *) map
    print
```

### Filtering even numbers

```
[1 2 3 4 5 6]
    (2 % 0 =) filter
    print
```

### Computing the sum of all even square numbers in a list

```
[1 2 3 4 5 6 7 8 9 10]
    (dup *) map
    (2 % 0 =) filter
    (+) 0 fold
    print
```

## Planned Features

- [ ] Recursion
- [ ] Modules
- [ ] Native exe compilation