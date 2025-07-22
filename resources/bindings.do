5 let x { x x * print }

10 2 let a b { a b / print }
10 2 / print

5 6
let x { x + }
print

4 5 let a b { a b * print }
4 5 let a b { a b - print }

4 5 let a b {
    a b + let sum {
        a b * let prod {
            sum print
            prod print
        }
    }
}

//Somehow adding some let bindings has made this example work
[1 2 3 4 5]
    (dup *) map
    print

(dup *) [5 6 7 8 9 10] let f l {
    l f map
}
print