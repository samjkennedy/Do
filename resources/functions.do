fn add_one (1 +)
10 add_one print

fn square (dup *)
5 square print

[1 2 3 4 5]
    (square) map
    print

fn square_list ((dup *) map)

[1 2 3 4 5]
    square_list print

fn all? ((and) true fold)
fn any? ((or) false fold)

[true true true] all?
    print

[false true true] all?
    print