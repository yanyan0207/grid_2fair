//! 1 行をビット列で表したときの共通の操作
//!
//! 列 c を塗ったら c ビット目を 1 にする。状態は直前の 2 行 (上の行, 今の行) で、
//! dp.rs と dfs.rs の両方が使う。

/// mask の部分集合のうち、1 の数が limit 個以下のものを全部 f に渡す（空集合から始め、深さ優先順）。
/// 次の行の候補は「強制マス ∪ 今の行の塗ったマスの真下の部分集合」で、部分集合で足すマスは
/// 1 個ずつ塗り数を増やすので、予算の余裕の個数までの部分集合だけを作ればよい
pub fn for_each_subset_upto(mask: u64, limit: usize, f: &mut impl FnMut(u64)) {
    fn go(bits: &[u64], start: usize, chosen: u64, left: usize, f: &mut impl FnMut(u64)) {
        f(chosen);
        if left == 0 {
            return;
        }
        for i in start..bits.len() {
            go(bits, i + 1, chosen | bits[i], left - 1, f);
        }
    }
    let mut bits = [0u64; 64];
    let mut len = 0;
    let mut m = mask;
    while m != 0 {
        bits[len] = m.isolate_lowest_one();
        m &= m - 1;
        len += 1;
    }
    go(&bits[..len], 0, 0, limit, f);
}

/// 行 cur の塗っていないマスが条件を満たすために、
/// 真下に塗りが必要なマスのビット列。満たせないマスがあれば None。
/// full は盤面幅ぶんのビットが立ったマスク
pub fn forced_below(full: u64, up: u64, cur: u64) -> Option<u64> {
    let a = up;
    let b = (cur << 1) & full;
    let c = cur >> 1;
    let all3 = a & b & c;
    let ones = (a ^ b ^ c) & !all3;
    let twos = ((a & b) | (b & c) | (a & c)) & !all3;
    let unpainted = !cur & full;
    if unpainted & !(ones | twos) != 0 {
        return None;
    }
    Some(unpainted & ones)
}

/// 状態のキー（上の行, 今の行）
pub fn key(up: u64, cur: u64) -> u64 {
    up << 32 | cur
}

/// key の逆。(上の行, 今の行) に戻す
pub fn split(st: u64) -> (u64, u64) {
    (st >> 32, st & 0xFFFF_FFFF)
}

/// 幅 n の行を左右反転する
pub fn mirror(n: usize, row: u64) -> u64 {
    row.reverse_bits() >> (64 - n)
}

/// 状態とその左右反転のうち、キーが小さい方。
/// 左右反転した状態は、その先の探索も鏡写しで最小塗り数も同じなので、1 つにまとめてよい
pub fn canonical(n: usize, up: u64, cur: u64) -> u64 {
    key(up, cur).min(key(mirror(n, up), mirror(n, cur)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mirror_roundtrip() {
        for n in 1..=30 {
            let full = (1u64 << n) - 1;
            for row in [0, 1, full, full >> 1, 0b1011 & full] {
                assert_eq!(mirror(n, mirror(n, row)), row, "n = {n}, row = {row:b}");
                assert_eq!(
                    mirror(n, row).count_ones(),
                    row.count_ones(),
                    "n = {n}, row = {row:b}"
                );
            }
            // 左端の 1 ビットは右端に移る
            assert_eq!(mirror(n, 1), 1 << (n - 1), "n = {n}");
        }
    }

    #[test]
    fn key_split_roundtrip() {
        for (up, cur) in [(0, 0), (1, 2), ((1 << 30) - 1, 5)] {
            assert_eq!(split(key(up, cur)), (up, cur));
        }
    }
}
