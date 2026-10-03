use crate::Board;

/// 塗るマス数の少ない順に全パターンを調べ、最小解を 1 つ返す。
/// 2^(n*n) 通りを調べるので n <= 5 程度まで
pub fn solve(n: usize) -> Board {
    assert!(n >= 1 && n * n < 64, "n too large for brute force");
    let cells = n * n;
    let limit = 1u64 << cells;
    for k in 0..=cells as u32 {
        if let Some(mask) = (0..limit)
            .filter(|m| m.count_ones() == k)
            .find(|&m| Board::from_mask(n, m).is_valid())
        {
            return Board::from_mask(n, mask);
        }
    }
    unreachable!("全マス塗れば必ず条件を満たす")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::known;

    #[test]
    fn matches_known_small() {
        for n in 1..=4 {
            let b = solve(n);
            assert!(b.is_valid());
            assert_eq!(Some(b.count()), known(n), "n = {n}");
        }
    }
}
