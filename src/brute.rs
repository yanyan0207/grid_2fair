use crate::Board;

/// 塗るマス数の少ない順に全パターンを調べ、最小解を 1 つ返す。
/// 塗り数 k のマスクだけを順に列挙する（Gosper's hack）。
/// それでも 2^(n*n) 通りの大半を調べるので n <= 5 程度まで
pub fn solve(n: usize) -> Board {
    assert!(n >= 1 && n * n < 64, "n too large for brute force");
    let cells = n * n;
    let limit = 1u64 << cells;
    for k in 0..=cells {
        // 塗り数 k で最小のマスクから始め、同じ塗り数の次に大きいマスクへ進む
        let mut m = (1u64 << k) - 1;
        while m < limit {
            if Board::from_mask(n, m).is_valid() {
                return Board::from_mask(n, m);
            }
            if m == 0 {
                break;
            }
            let low = m.isolate_lowest_one();
            let ripple = m + low;
            m = (((ripple ^ m) >> 2) / low) | ripple;
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
