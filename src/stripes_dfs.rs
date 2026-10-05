//! 縞の行で解を作り、DFS で最小だと証明する（--algo stripes-dfs）
//!
//! dfs.rs は予算を下界から 1 つずつ上げるが、ここでは逆に上から下ろす。
//! 1. 端以外の行を縞模様に制限して（stripes.rs）、予算以内の解を 1 つ作る（construct）。
//!    最適解は内側が周期 3 の斜めの縞になっているので、制限しても最小の解が得られることが多い。
//!    その塗り数を U とする。
//! 2. 制限なしの DFS で、予算 U-1 で塗れるかを調べる。塗れなければ a(n) = U が確定する
//!    （U-1 個で塗れないなら、それより少なくても塗れない）。
//!    塗れたら、その解の塗り数を新しい U にして 2 を繰り返す。
//!
//! 制限は解を作る側にしか使わないので、結論は制限のない探索と同じく厳密。
//!
//! construct は、真ん中の 2 行を縞の行から選び、そこから下の端へ、続けて上の端へ伸ばす DFS。
//! 予算以内の縞の解が 1 つ見つかった時点で止める。結果を覚える表は使わないので、メモリは経路の分だけ。

use crate::Board;
use crate::bound::{lower_bounds, strip_height};
use crate::dfs::{Dfs, trivial, upper_bound};
use crate::row::forced_below;
use crate::stripes::{DEFAULT_MARGIN, Stripes};

/// どちらの段階か
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// 縞に制限して解を作る。予算の代わりに作った解の塗り数を渡す
    Construct,
    /// 制限なしで、見つけた解より 1 少ない予算では塗れないことを確かめる
    Prove,
}

/// Construct::extend の 1 段で、次の行を選ぶときに使う値
struct Step {
    cur: u64,
    forced: u64,
    /// ここまでの塗り数 + forced の数
    base: usize,
    rem: usize,
    reserve: usize,
    downward: bool,
}

/// 縞の解を探す DFS。真ん中の 2 行 (a, b) から下の端まで伸ばし、続けて上の端まで伸ばす。
/// 結果を覚える表は使わず、予算以内の解が 1 つ見つかった時点で止める
struct Construct {
    n: usize,
    full: u64,
    stripes: Stripes,
    /// lb[m]: 端に接する m 行に最低限必要な塗り数（bound::lower_bounds の anchored。上下反転しても同じ）
    lb: Vec<usize>,
    budget: usize,
    /// 真ん中の 2 行と、上側の行数
    a: u64,
    b: u64,
    above: usize,
    /// 行 r+2, r+3, ... と下へ伸ばした行
    down: Vec<u64>,
    /// 行 r-1, r-2, ... と上へ伸ばした行
    up: Vec<u64>,
    nodes: u64,
}

impl Construct {
    /// 行 prev, cur の先（端の方向）に rem 行を伸ばす。cost はここまでの塗り数、
    /// reserve はまだ伸ばしていない反対側に最低限必要な塗り数。
    /// downward なら下側（終わったら上側に移る）、そうでなければ上側
    fn extend(
        &mut self,
        prev: u64,
        cur: u64,
        rem: usize,
        cost: usize,
        reserve: usize,
        downward: bool,
    ) -> bool {
        self.nodes += 1;
        let Some(forced) = forced_below(self.full, prev, cur) else {
            return false;
        };
        if rem == 0 {
            // cur が端の行。その先は盤の外なので、強制マスがあってはいけない
            if forced != 0 {
                return false;
            }
            if downward {
                // 下側が終わったので、盤を上下反転して行 r+1, r = b, a から上へ伸ばす
                let (a, b, above) = (self.a, self.b, self.above);
                return self.extend(b, a, above, cost, 0, false);
            }
            return true;
        }
        // 次の行は forced ∪ (cur の部分集合)。部分集合で足すマスは予算の余裕の個数までしか足せないので、
        // 余裕を先に求め、その個数までの部分集合だけを作る
        let base = cost + forced.count_ones() as usize;
        let Some(slack) = self.budget.checked_sub(base + reserve + self.lb[rem - 1]) else {
            return false;
        };
        let mut bits = [0u64; 64];
        let mut len = 0;
        let mut m = cur;
        while m != 0 {
            bits[len] = m.isolate_lowest_one();
            m &= m - 1;
            len += 1;
        }
        let ctx = Step {
            cur,
            forced,
            base,
            rem,
            reserve,
            downward,
        };
        self.choose(&ctx, &bits[..len], 0, 0, slack)
    }

    /// extend の続き。cur の塗ったマスの真下のうち bits[start..] から、あと left 個まで選んで足す。
    /// 選んだ組み合わせ chosen ごとに、次の行 forced ∪ chosen を試す
    fn choose(&mut self, s: &Step, bits: &[u64], start: usize, chosen: u64, left: usize) -> bool {
        let next = s.forced | chosen;
        let c2 = s.base + chosen.count_ones() as usize;
        // next の先には rem-1 行ある。端から margin 行以内なら自由、それより内側なら縞に限る
        let allowed = s.rem - 1 < self.stripes.margin() || self.stripes.is_striped(next);
        if allowed && self.fits(s.cur, next, s.rem - 1, c2 + s.reserve) {
            if s.downward {
                self.down.push(next);
            } else {
                self.up.push(next);
            }
            if self.extend(s.cur, next, s.rem - 1, c2, s.reserve, s.downward) {
                return true;
            }
            if s.downward {
                self.down.pop();
            } else {
                self.up.pop();
            }
        }
        if left > 0 {
            for i in start..bits.len() {
                if self.choose(s, bits, i + 1, chosen | bits[i], left - 1) {
                    return true;
                }
            }
        }
        false
    }

    /// 行 cur の次に next を置いたとき、その先の rem 行を含めて予算に収まる見込みがあるか（1 行先読み）。
    /// total はここまでの塗り数と反対側の見積もりの和
    fn fits(&self, cur: u64, next: u64, rem: usize, total: usize) -> bool {
        let Some(forced) = forced_below(self.full, cur, next) else {
            return false;
        };
        if rem == 0 {
            return forced == 0 && total <= self.budget;
        }
        total + self.lb[rem].max(forced.count_ones() as usize + self.lb[rem - 1]) <= self.budget
    }
}

/// 端以外の行（上下 margin 行と、行の左右 margin 列を除く部分）を縞に制限した解を 1 つ作る。
/// 予算は証明済みの上限 ceiling((n+2)^2/3) - 6 から始め、縞の解が見つからなければ 1 つずつ上げる
/// （上限を示す構成の解も縞の形なので、n >= 7 では最初の予算で見つかるはず）。
/// 結果を覚える表は使わないので、メモリは経路の分だけで済む。
/// 戻り値は (盤面, 探索ノード数)。盤が小さくて真ん中に縞の行を 2 行置けない、
/// または自明な解の個数より少ない予算で見つからないときは None
pub fn construct(n: usize, margin: usize) -> Option<(Board, usize)> {
    let stripes = Stripes::new(n, margin)?;
    if n < 2 * margin + 2 {
        return None;
    }
    // 真ん中の 2 行 r, r+1。どちらも端から margin 行より内側にある
    let r = n / 2 - 1;
    let (above, below) = (r, n - 2 - r);
    let mut candidates = stripes.rows();
    // 塗り数の少ない行から試す
    candidates.sort_by_key(|row| row.count_ones());
    let mut s = Construct {
        n,
        full: (1u64 << n) - 1,
        stripes,
        lb: lower_bounds(n, strip_height(n)).anchored,
        budget: 0,
        a: 0,
        b: 0,
        above,
        down: Vec::with_capacity(below),
        up: Vec::with_capacity(above),
        nodes: 0,
    };
    let start = ((n + 2) * (n + 2))
        .div_ceil(3)
        .saturating_sub(6)
        .max(s.lb[n]);
    for budget in start..upper_bound(n) {
        s.budget = budget;
        for &a in &candidates {
            for &b in &candidates {
                let base = (a.count_ones() + b.count_ones()) as usize;
                if base + s.lb[below] + s.lb[above] > budget {
                    continue;
                }
                (s.a, s.b) = (a, b);
                s.down.clear();
                s.up.clear();
                // 下側: 行 r, r+1 = a, b から下へ。上側に最低限必要な数は予算から差し引いておく
                let reserve = s.lb[above];
                if s.extend(a, b, below, base, reserve, true) {
                    let mut rows: Vec<u64> = s.up.iter().rev().copied().collect();
                    rows.push(a);
                    rows.push(b);
                    rows.extend(s.down.iter().copied());
                    let mut board = Board::new(s.n);
                    for (i, &row) in rows.iter().enumerate() {
                        for c in 0..s.n {
                            board.painted[i * s.n + c] = row >> c & 1 == 1;
                        }
                    }
                    return Some((board, s.nodes as usize));
                }
            }
        }
    }
    None
}

/// 最小解と探索ノード数（解を作る DFS と、証明の DFS の合計）を返す。
/// margin は縞に制限するときに自由にする上下の行数と左右の列数。
/// 段階ごとに on_budget(段階, 予算, 塗れたか, 探索ノード数) を呼ぶ
pub fn solve_by_budget(
    n: usize,
    margin: usize,
    mut on_budget: impl FnMut(Phase, usize, bool, u64),
) -> (Board, u64) {
    // 1. 縞に制限して解を作る。作れなければ自明な解から始める
    let (mut best, construct_nodes) = match construct(n, margin) {
        Some((board, nodes)) => (board, nodes as u64),
        None => (trivial(n), 0),
    };
    on_budget(Phase::Construct, best.count(), true, construct_nodes);
    // 2. 制限なしで、見つけた解より 1 少ない予算では塗れないことを確かめる
    let mut prove = Dfs::new(n);
    while let Some(budget) = best.count().checked_sub(1) {
        let before = prove.nodes();
        let found = prove.try_budget(budget);
        on_budget(
            Phase::Prove,
            budget,
            found.is_some(),
            prove.nodes() - before,
        );
        match found {
            Some(board) => best = board,
            None => break,
        }
    }
    (best, construct_nodes + prove.nodes())
}

/// 既定の制限の幅で、最小解を 1 つ返す
pub fn solve(n: usize) -> Board {
    solve_by_budget(n, DEFAULT_MARGIN, |_, _, _, _| {}).0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::known;

    #[test]
    fn matches_known() {
        for n in 1..=15 {
            let b = solve(n);
            assert!(b.is_valid(), "n = {n}");
            assert_eq!(Some(b.count()), known(n), "n = {n}");
        }
    }

    /// 制限の幅を変えても答えは変わらない（0 なら縞の制限が一番きつい）
    #[test]
    fn any_margin_gives_same_answer() {
        for margin in 0..=4 {
            for n in 1..=12 {
                let (b, _) = solve_by_budget(n, margin, |_, _, _, _| {});
                assert!(b.is_valid(), "n = {n}, margin = {margin}");
                assert_eq!(Some(b.count()), known(n), "n = {n}, margin = {margin}");
            }
        }
    }

    /// 作った解は条件を満たし、制限のない最小以上で、証明済みの上限以下
    #[test]
    fn construct_is_valid() {
        for n in 6..=15 {
            let (b, _) = construct(n, DEFAULT_MARGIN).expect("striped solutions exist");
            assert!(b.is_valid(), "n = {n}");
            assert!(Some(b.count()) >= known(n), "n = {n}");
            assert!(
                b.count() <= ((n + 2) * (n + 2)).div_ceil(3) - 6 || n < 7,
                "n = {n}"
            );
        }
    }
}
