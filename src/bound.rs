//! 残りの行に最低限必要な塗り数（下界）
//!
//! 1 行だけでは下界にならない（横縞なら空の行は上下の塗りで条件を満たす）ので、
//! 高さ h の帯の最小塗り数（strip_min）を列方向の DP で求め、帯をつないで下界にする。
//! dp.rs と dfs.rs の両方が、この下界で枝を切る。

/// 帯の高さの上限。状態数 4^h の列方向 DP を回すので 9 程度まで
pub const STRIP_HEIGHT: usize = 9;

/// 高さ h・幅 n の帯で、帯の全マスが条件を満たすときの最小塗り数。
/// top_free / bot_free が true なら帯の上 / 下の外側の行は好きに塗れるとみなし、
/// false なら盤の外（塗れない）とみなす。左右は盤の端。
///
/// 左から 1 列ずつ決める DP。状態は (左の列, 今の列)。
/// 今の列の塗っていないマスについて、右の列のマスを塗る必要があるか
/// （外側が自由ならどちらでもよいか）を求めて遷移する。
pub fn strip_min(n: usize, h: usize, top_free: bool, bot_free: bool) -> usize {
    assert!(n >= 1 && (1..=12).contains(&h));
    const INF: u16 = u16::MAX;
    let size = 1usize << h;
    let mask = size - 1;
    let mut cur = vec![INF; size * size];
    for (c, v) in cur.iter_mut().enumerate().take(size) {
        // 左の列は盤の外（塗れない）
        *v = c.count_ones() as u16;
    }
    for col in 0..n {
        let last = col + 1 == n;
        let mut next = if last {
            Vec::new()
        } else {
            vec![INF; size * size]
        };
        let mut best = INF;
        for l in 0..size {
            for c in 0..size {
                let v = cur[l * size + c];
                if v == INF {
                    continue;
                }
                let Some((must1, must0)) = column_requirements(h, l, c, top_free, bot_free) else {
                    continue;
                };
                if last {
                    // 右の列は盤の外なので塗れない
                    if must1 == 0 {
                        best = best.min(v);
                    }
                    continue;
                }
                let free = mask & !must1 & !must0;
                let mut sub = free;
                loop {
                    let r = must1 | sub;
                    let w = v + r.count_ones() as u16;
                    let k = c * size + r;
                    if w < next[k] {
                        next[k] = w;
                    }
                    if sub == 0 {
                        break;
                    }
                    sub = (sub - 1) & free;
                }
            }
        }
        if last {
            assert!(best != INF, "the strip has no solution");
            return best as usize;
        }
        cur = next;
    }
    unreachable!()
}

/// 列 c の塗っていないマスが条件を満たすために、右の列 r が満たすべき条件。
/// (必ず塗るマス, 塗ってはいけないマス)。満たせないマスがあれば None
fn column_requirements(
    h: usize,
    l: usize,
    c: usize,
    top_free: bool,
    bot_free: bool,
) -> Option<(usize, usize)> {
    let (mut must1, mut must0) = (0, 0);
    for i in 0..h {
        if c >> i & 1 == 1 {
            continue;
        }
        // s: 決まっている塗りの数、f: 好きに塗れる外側の隣の数
        let mut s = l >> i & 1;
        let mut f = 0;
        if i > 0 {
            s += c >> (i - 1) & 1;
        } else if top_free {
            f += 1;
        }
        if i + 1 < h {
            s += c >> (i + 1) & 1;
        } else if bot_free {
            f += 1;
        }
        // 右のマスを x にしたとき、残り 2 - s - x 個を外側でまかなえるか
        let ok = |x: usize| s + x <= 2 && 2 - s - x <= f;
        match (ok(0), ok(1)) {
            (false, false) => return None,
            (false, true) => must1 |= 1 << i,
            (true, false) => must0 |= 1 << i,
            (true, true) => {}
        }
    }
    Some((must1, must0))
}

/// 残りの行に最低限必要な塗り数（いずれも m = 0..=n で引く）
pub struct Bounds {
    /// anchored[m]: 盤の下端に接する m 行に最低限必要な塗り数
    pub anchored: Vec<usize>,
    /// free[m]: 上下の外側を自由とみなした連続する m 行に最低限必要な塗り数
    pub free: Vec<usize>,
}

/// 下界を作る。下端に接する高さ h の帯（strip_min(.., true, false)）と、
/// 外側が両方自由な帯（strip_min(.., true, true)）に分けた和の最大
pub fn lower_bounds(n: usize, max_h: usize) -> Bounds {
    let max_h = max_h.min(n);
    let strip = |h: usize, bot_free: bool| {
        if h == 0 {
            0
        } else {
            strip_min(n, h, true, bot_free)
        }
    };
    let free_strip: Vec<usize> = (0..=max_h).map(|h| strip(h, true)).collect();
    let bottom_strip: Vec<usize> = (0..=max_h).map(|h| strip(h, false)).collect();
    let mut free = vec![0; n + 1];
    for j in 1..=n {
        free[j] = (1..=max_h.min(j))
            .map(|t| free_strip[t] + free[j - t])
            .max()
            .unwrap();
    }
    let mut anchored = vec![0; n + 1];
    for m in 1..=n {
        anchored[m] = (1..=max_h.min(m))
            .map(|h| bottom_strip[h] + free[m - h])
            .max()
            .unwrap();
    }
    Bounds { anchored, free }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::known;

    #[test]
    fn strip_min_small() {
        // 1 行で外側が両方自由なら、全マスを上下でまかなえる
        assert_eq!(strip_min(5, 1, true, true), 0);
        // 2 行あれば、帯の中に塗りが必要
        assert!(strip_min(5, 2, true, true) > 0);
        // 帯の高さ = n で外側が両方盤の外なら a(n) そのもの
        for n in 1..=6 {
            assert_eq!(Some(strip_min(n, n, false, false)), known(n), "n = {n}");
        }
    }

    #[test]
    fn lower_bound_is_below_known() {
        for n in 1..=10 {
            let lb = lower_bounds(n, STRIP_HEIGHT);
            assert!(lb.anchored[n] <= known(n).unwrap(), "n = {n}");
        }
    }
}
