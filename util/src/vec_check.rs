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

/// 检查两个Vector是否相等
pub fn is_equal(a: &Vec<usize>, b: &Vec<usize>) -> bool {
    let mut a_ = a.clone();
    let mut b_ = b.clone();
    // 排序后去除重复元素
    a_.sort();
    a_.dedup();
    b_.sort();
    b_.dedup();
    
    if a_.len() != b_.len() {
        return false;
    }
    for i in 0..a_.len() {
        if a_[i] != b_[i] {
            return false;
        }
    }
    true
}