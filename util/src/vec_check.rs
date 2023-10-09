/// 检查一个a: Vector是否为另一个b: Vector的子集
pub fn is_subset<T: PartialEq>(a: &Vec<T>, b: &Vec<T>) -> bool {
    for i in a {
        if !b.contains(i) {
            return false;
        }
    }
    true
}

/// 检查一个元素是否在一个Vector中
pub fn is_invector<T: PartialEq>(a: T, b: &Vec<T>) -> bool {
    for i in b {
        if *i == a {
            return true;
        }
    }
    false
}