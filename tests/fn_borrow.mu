fn borrow ( ref y : box int ) {
    let mut z = y
}

let mut x = box 0

borrow ( x )
let mut y = ref x