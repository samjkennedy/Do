true (10 print) if
false (10 print) if

false (1) (2) choice print
[1 2 3 4 5 6 7 8 9 10]
    (dup 2 % 0 = (1 print) if pop) foreach

 //TODO: The type checker doesn't support choice inside inferred functions
//[1 2 3 4 5]
//    (dup 2 % 0 = (dup *) (2 *) choice) map
//    print

//TODO: The type checker expects the if body to be of type [ -- ] but it should allow any "symmetrical" function
[1 2 3 4 5 6 7 8 9 10]
    (dup 2 % 0 = (1 print) if pop) foreach