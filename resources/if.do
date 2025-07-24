true if { 3 print }
false if { 4 print } else { 5 print}

false let b {
    10 b if {
        10 +
    } else {
        true if {
            15 +
        } else {
            5 +
        }
    }
}
print

//TODO some way to handle nested ifs...

5 dup 0 = if {
    1 print
} else {
    dup 1 = if {
        2 print
    } else {
        dup 2 = if {
            3 print
        } else {
            4 print
        }
    }
}
pop