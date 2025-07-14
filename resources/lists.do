[1 2 3 4 5] print
[[1 2] [3 4]] print

[(1 dup * print) (2 dup * print) (3 dup * print)]
    (do) foreach

[1 2 3 4 5]
    (+) 0 fold
    print

[[2 4] [5 6 7 8] [12 10]]
    ((2 % 0 =) map) map
    ((and) true fold) map
    (.) filter len
    print

[[1 2] [3 4] [5 6]]
    (concat) [] fold
    print
