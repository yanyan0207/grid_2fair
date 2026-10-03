//! 予算付きの行単位 DP（下界による枝刈り）
//!
//! 状態は「直前の 2 行」(up, cur)。行 r から先の最小値はこの 2 行だけで決まるので、
//! 上から 1 行ずつ、到達できる状態とそこまでの最小塗り数を更新していく。
//!
//! 予算 budget を決めて「budget 個以下で塗れるか」を判定する。
//! 行 0..=k の塗り数 c に、残りの行に最低限必要な塗り数 lb[n-1-k] を足して
//! 予算を超える状態は捨てる。下界は高さ h の帯の最小値（strip_min）から作る。
//!
//! 予算を下界から 1 つずつ上げ、最初に解が見つかった予算が a(n)。
//! それより小さい予算では「解なし」が示されたことになる。

use std::collections::HashMap;

use crate::Board;
use crate::dfs::forced_below;

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
            assert!(best != INF, "帯の解が存在しない");
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

/// lb[m]: 盤の下端に接する m 行に最低限必要な塗り数（m = 0..=n）。
/// 下端に接する高さ h の帯（strip_min(.., true, false)）と、
/// 外側が両方自由な帯（strip_min(.., true, true)）に分けた和の最大
pub fn lower_bounds(n: usize, max_h: usize) -> Vec<usize> {
    let max_h = max_h.min(n);
    let free: Vec<usize> = (0..=max_h)
        .map(|h| {
            if h == 0 {
                0
            } else {
                strip_min(n, h, true, true)
            }
        })
        .collect();
    let bottom: Vec<usize> = (0..=max_h)
        .map(|h| {
            if h == 0 {
                0
            } else {
                strip_min(n, h, true, false)
            }
        })
        .collect();
    // g[j]: 外側自由な帯だけで j 行を分けたときの和の最大
    let mut g = vec![0; n + 1];
    for j in 1..=n {
        g[j] = (1..=max_h.min(j))
            .map(|t| free[t] + g[j - t])
            .max()
            .unwrap();
    }
    let mut lb = vec![0; n + 1];
    for m in 1..=n {
        lb[m] = (1..=max_h.min(m))
            .map(|h| bottom[h] + g[m - h])
            .max()
            .unwrap();
    }
    lb
}

/// 1 回の予算判定の結果
pub struct Outcome {
    /// 予算以内の最小解。なければ None
    pub board: Option<Board>,
    /// 遷移の数
    pub transitions: u64,
    /// 1 行あたりの状態数の最大
    pub max_states: usize,
}

/// 状態のキー（上の行, 今の行）
fn key(up: u64, cur: u64) -> u64 {
    up << 32 | cur
}

/// budget 個以下で塗れるかを判定し、塗れるなら最小解を返す
pub fn search(n: usize, budget: usize, lb: &[usize]) -> Outcome {
    assert!((1..=30).contains(&n), "n must be in 1..=30");
    let full = (1u64 << n) - 1;
    let keep = |k: usize, c: usize| c + lb[n - 1 - k] <= budget;

    // layers[k]: 行 k までの状態 → (最小塗り数, 1 つ前の状態の上の行)
    let mut layers: Vec<HashMap<u64, (u16, u32)>> = Vec::with_capacity(n);
    let mut first = HashMap::new();
    for row in 0..=full {
        let c = row.count_ones() as usize;
        if keep(0, c) {
            first.insert(key(0, row), (c as u16, 0));
        }
    }
    layers.push(first);

    let mut transitions = 0u64;
    let mut best: Option<(u16, u64)> = None;
    for k in 0..n {
        let mut next = HashMap::new();
        for (&st, &(v, _)) in &layers[k] {
            let (up, cur) = (st >> 32, st & 0xFFFF_FFFF);
            let Some(forced) = forced_below(full, up, cur) else {
                continue;
            };
            if k + 1 == n {
                // 盤外の行は塗れない
                if forced == 0 && best.is_none_or(|(b, _)| v < b) {
                    best = Some((v, st));
                }
                continue;
            }
            let mut sub = cur;
            loop {
                let down = forced | sub;
                let w = v + down.count_ones() as u16;
                transitions += 1;
                if keep(k + 1, w as usize) {
                    let e = next.entry(key(cur, down)).or_insert((u16::MAX, 0));
                    if w < e.0 {
                        *e = (w, up as u32);
                    }
                }
                if sub == 0 {
                    break;
                }
                sub = (sub - 1) & cur;
            }
        }
        if k + 1 < n {
            layers.push(next);
        }
    }
    let max_states = layers.iter().map(HashMap::len).max().unwrap_or(0);

    let board = best.map(|(_, mut st)| {
        // 後ろから親をたどって各行を復元する
        let mut rows = vec![0u64; n];
        for k in (0..n).rev() {
            let (up, cur) = (st >> 32, st & 0xFFFF_FFFF);
            rows[k] = cur;
            let (_, parent) = layers[k][&st];
            st = key(parent as u64, up);
        }
        let mut board = Board::new(n);
        for (r, &row) in rows.iter().enumerate() {
            for c in 0..n {
                board.painted[r * n + c] = row >> c & 1 == 1;
            }
        }
        board
    });
    Outcome {
        board,
        transitions,
        max_states,
    }
}

/// 予算を下界から 1 つずつ上げ、最初に解が見つかった予算で最小解を返す。
/// on_budget は各予算の判定結果ごとに呼ばれる
pub fn solve_by_budget(n: usize, mut on_budget: impl FnMut(usize, &Outcome)) -> Board {
    let lb = lower_bounds(n, STRIP_HEIGHT);
    for budget in lb[n].. {
        let outcome = search(n, budget, &lb);
        on_budget(budget, &outcome);
        if let Some(board) = outcome.board {
            return board;
        }
    }
    unreachable!()
}

pub fn solve(n: usize) -> Board {
    solve_by_budget(n, |_, _| {})
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{dfs, known};

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
            assert!(lb[n] <= known(n).unwrap(), "n = {n}");
        }
    }

    #[test]
    fn matches_dfs() {
        for n in 1..=6 {
            assert_eq!(solve(n).count(), dfs::solve(n).count(), "n = {n}");
        }
    }

    #[test]
    fn matches_known() {
        for n in 1..=10 {
            let b = solve(n);
            assert!(b.is_valid(), "n = {n}");
            assert_eq!(Some(b.count()), known(n), "n = {n}");
        }
    }

    #[test]
    fn budget_below_answer_is_infeasible() {
        for n in 1..=10 {
            let lb = lower_bounds(n, STRIP_HEIGHT);
            let a = known(n).unwrap();
            assert!(search(n, a - 1, &lb).board.is_none(), "n = {n}");
        }
    }
}
