 true (10 print) if
 false (10 print) if

false (1) (2) choice print

[1 2 3 4 5]
    //TODO: The type checker doesn't seem happy with this
    (dup 2 % 0 = (dup *) (2 *) choice) map
    print
