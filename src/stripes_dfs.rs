//! 縞の行で解を作り、DFS で最小だと証明する（--algo stripes-dfs）
//!
//! dfs.rs は予算を下界から 1 つずつ上げるが、ここでは逆に上から下ろす。
//! 1. 端以外の行を縞模様に制限して（stripes.rs）、その中で最小の解を作る（construct）。
//!    最適解は内側が周期 3 の斜めの縞になっているので、制限しても最小かそれに近い解が得られる。
//!    その塗り数を U とする。
//! 2. 制限なしの DFS で、予算 U-1 で塗れるかを調べる。塗れなければ a(n) = U が確定する
//!    （U-1 個で塗れないなら、それより少なくても塗れない）。
//!    塗れたら、その解の塗り数を新しい U にして 2 を繰り返す。
//!
//! 制限は解を作る側にしか使わないので、結論は制限のない探索と同じく厳密。
//!
//! construct は、真ん中の 2 行を縞の行から選び、そこから上下の端へ向かって探す。
//! 2 行が決まれば、上側の行の条件と下側の行の条件は互いに関係しないので、別々に最小を求めて足す。
//! 上側も下側も「決まった 2 行から端まで、何行残っているか」だけで決まる同じ問題なので
//! （盤は上下反転しても同じ問題）、同じ関数 Half::best で求め、結果を覚えて使い回す。

use rustc_hash::FxHashMap as HashMap;

use crate::Board;
use crate::dfs::{Dfs, trivial};
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

/// 決まった 2 行から端へ向かって、残りの行を縞の制限のもとで最小に塗る
struct Half {
    full: u64,
    stripes: Stripes,
    /// (上の行, 今の行, 残りの行数) → (残りの行の最小塗り数, そのときの次の行)。塗れなければ None
    memo: HashMap<(u64, u64, usize), Option<(usize, u64)>>,
}

impl Half {
    /// 行 up, cur の先（端の方向）に rem 行を塗るときの最小塗り数。
    /// cur と、新しく塗る行がすべて条件を満たし、最後の行の先（盤の外）に強制マスがないこと。
    /// 新しく塗る行のうち、端から margin 行より内側の行は縞の行に限る
    fn best(&mut self, up: u64, cur: u64, rem: usize) -> Option<usize> {
        if let Some(&v) = self.memo.get(&(up, cur, rem)) {
            return v.map(|(c, _)| c);
        }
        let v = self.compute(up, cur, rem);
        self.memo.insert((up, cur, rem), v);
        v.map(|(c, _)| c)
    }

    fn compute(&mut self, up: u64, cur: u64, rem: usize) -> Option<(usize, u64)> {
        let forced = forced_below(self.full, up, cur)?;
        if rem == 0 {
            // cur が端の行。その先は盤の外なので塗れない
            return (forced == 0).then_some((0, 0));
        }
        let mut best: Option<(usize, u64)> = None;
        // 次の行は forced ∪ (cur の部分集合)
        let mut sub = cur;
        loop {
            let down = forced | sub;
            // down の先には rem-1 行ある。端から margin 行以内なら自由、それより内側なら縞に限る
            let allowed = rem - 1 < self.stripes.margin() || self.stripes.is_striped(down);
            if allowed && let Some(c) = self.best(cur, down, rem - 1) {
                let c = c + down.count_ones() as usize;
                if best.is_none_or(|(b, _)| c < b) {
                    best = Some((c, down));
                }
            }
            if sub == 0 {
                break;
            }
            sub = (sub - 1) & cur;
        }
        best
    }

    /// best で求めた最小の塗り方の行を、端に向かう順に返す
    fn path(&mut self, mut up: u64, mut cur: u64, mut rem: usize) -> Vec<u64> {
        let mut rows = Vec::with_capacity(rem);
        while rem > 0 {
            self.best(up, cur, rem);
            let (_, down) =
                self.memo[&(up, cur, rem)].expect("path is only asked for solvable states");
            rows.push(down);
            (up, cur, rem) = (cur, down, rem - 1);
        }
        rows
    }
}

/// 端以外の行（上下 margin 行と、行の左右 margin 列を除く部分）を縞に制限した解のうち、最小のもの。
/// 戻り値は (盤面, 覚えた状態の数)。制限のもとで解がない、または盤が小さくて真ん中に縞の行を
/// 2 行置けないときは None
pub fn construct(n: usize, margin: usize) -> Option<(Board, usize)> {
    let stripes = Stripes::new(n, margin)?;
    if n < 2 * margin + 2 {
        return None;
    }
    // 真ん中の 2 行 r, r+1。どちらも端から margin 行より内側にある
    let r = n / 2 - 1;
    let (above, below) = (r, n - 2 - r);
    let candidates = stripes.rows();
    let mut half = Half {
        full: (1u64 << n) - 1,
        stripes,
        memo: HashMap::default(),
    };
    let mut best: Option<(usize, u64, u64)> = None;
    for &a in &candidates {
        for &b in &candidates {
            let base = (a.count_ones() + b.count_ones()) as usize;
            if best.is_some_and(|(c, _, _)| base >= c) {
                continue;
            }
            // 下側: 行 r, r+1 = a, b から下へ。上側: 盤を上下反転して、行 r+1, r = b, a から上へ
            let Some(bottom) = half.best(a, b, below) else {
                continue;
            };
            let Some(top) = half.best(b, a, above) else {
                continue;
            };
            let total = base + top + bottom;
            if best.is_none_or(|(c, _, _)| total < c) {
                best = Some((total, a, b));
            }
        }
    }
    let (_, a, b) = best?;
    let mut rows = half.path(b, a, above);
    rows.reverse();
    rows.push(a);
    rows.push(b);
    rows.extend(half.path(a, b, below));
    let mut board = Board::new(n);
    for (i, &row) in rows.iter().enumerate() {
        for c in 0..n {
            board.painted[i * n + c] = row >> c & 1 == 1;
        }
    }
    Some((board, half.memo.len()))
}

/// 最小解と探索ノード数（覚えた状態の数と、DFS の探索ノード数の合計）を返す。
/// margin は縞に制限するときに自由にする上下の行数と左右の列数。
/// 段階ごとに on_budget(段階, 予算, 塗れたか, 探索ノード数) を呼ぶ
pub fn solve_by_budget(
    n: usize,
    margin: usize,
    mut on_budget: impl FnMut(Phase, usize, bool, u64),
) -> (Board, u64) {
    // 1. 縞に制限して解を作る。作れなければ自明な解から始める
    let (mut best, construct_nodes) = match construct(n, margin) {
        Some((board, states)) => (board, states as u64),
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

    /// 作った解は条件を満たし、制限のない最小以上
    #[test]
    fn construct_is_valid() {
        for n in 6..=15 {
            let (b, _) = construct(n, DEFAULT_MARGIN).expect("striped solutions exist");
            assert!(b.is_valid(), "n = {n}");
            assert!(Some(b.count()) >= known(n), "n = {n}");
        }
    }
}
