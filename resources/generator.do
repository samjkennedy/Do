//Hypothetical syntax for a "lazy list"
fn first_n (
    0
    let num n {
        num n < if {
            n yield
            n 1 + first_n
        }
        n return
    }
)