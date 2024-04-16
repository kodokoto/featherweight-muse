fn swap ( mut ref a : int , mut ref b : int ) {
    let mut c = deref a
    deref a = deref b
    deref b = c
} 

let mut x = 0
let mut y = 1

swap ( x , y )