fn test ( y : box int ) : int {
    y = box 6
    move y
}
let mut x = box 5
let mut p = test ( move x )

