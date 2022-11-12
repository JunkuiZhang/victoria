pub fn max_3number<T: PartialOrd>(n0: T, n1: T, n2: T) -> T {
    let mut max_num = n0;
    if n1 > max_num {
        max_num = n1;
    }
    if n2 > max_num {
        max_num = n2;
    }

    max_num
}

pub fn min_3number<T: PartialOrd>(n0: T, n1: T, n2: T) -> T {
    let mut min_num = n0;
    if n1 < min_num {
        min_num = n1;
    }
    if n2 < min_num {
        min_num = n2;
    }

    min_num
}
