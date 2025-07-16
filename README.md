# Do Programming Language

Do is a stack-based, functional programming language designed around composition, inference, and lists. The
concatenation of two programs is their composition.

It draws inspiration from Forth, Porth, and Factor and Joy, combining postfix syntax with Hindley-Milner-esque
type inference and first-class functions.

## Design Goals

- Understandable syntax, arbitrary symbols are avoided in favour of intuitive keywords where appropriate.
- First-class functions: Lambdas and Higher Order Functions are core.
- Static type checking with type inference.
- Interactive development via a built-in REPL.

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

### Conditionals

| Operation | Signature                                     | Description                                                        |
|-----------|-----------------------------------------------|--------------------------------------------------------------------|
| if        | bool fn(->) ->                                | Run block if condition is true                                     |
| choice    | ... bool fn(... -> ...) fn(... -> ...) -> ... | If/else: choose one of two branches (\<cond> (then) (else) choice) |

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
- [ ] WASM compilation