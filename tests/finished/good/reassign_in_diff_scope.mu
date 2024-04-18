fn setToOne ( mut ref a : int ) {
    deref a = 1
}

let mut x = 0
let mut y = x
setToOne ( y )
let mut w = y
