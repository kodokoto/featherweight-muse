
fn main() : box int {
    let mut x = box 5
    let mut y = x
    let mut z = y
    z
}

let mut res = main()
