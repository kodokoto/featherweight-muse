fn block ( mut ref y : int ) {
    let mut z = 1
    y = mut ref z
}

let mut x = 0
let mut y = mut ref x
block ( y )
let mut w = y
