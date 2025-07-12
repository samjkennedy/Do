
//A hypothetical syntax, does not compile

//Using let bindings to count the occurrences of a value in a list
[1 2 1 2 1 3]  // data
1              // needle
let x a (        // bind 1 to `x`, the list to `a`
  a
  (x =) map
  (true =) filter
  len print
)              // block is now on the stack - but not yet executed
do             // executes the block