//! 行単位の深さ優先探索（予算付きの分枝限定法）
//!
//! 各行をビット列で表す。行 r とその上の行 r-1 が決まると、
//! 行 r の塗っていないマスが必要とする「下の塗りマス」の数は
//! 2 - (上 + 左 + 右) に確定する。これが 0 か 1 でなければ枝刈り。
//! したがって行 r+1 は、行 r の塗っていないマスの真下が強制され、
//! 自由に選べるのは行 r の塗ったマスの真下だけになる。
//!
//! dp.rs と同じく、予算 budget を下界から 1 つずつ上げて「budget 個以下で塗れるか」を判定する。
//! 枝刈りも dp.rs と同じで、帯の下界（bound::lower_bounds）と 1 行先読みを使う。
//! 左右対称は、行 0 を左右反転したもの以下に限ることで使う。
//! DP と違って状態の表を持たないので、メモリは今の経路の n 行分だけで済む。

use crate::Board;
use crate::bound::{STRIP_HEIGHT, lower_bounds};
use crate::row::{forced_below, mirror};

pub struct Dfs {
    n: usize,
    full: u64,
    /// lb[m]: 盤の下端に接する m 行に最低限必要な塗り数
    lb: Vec<usize>,
    budget: usize,
    rows: Vec<u64>,
    nodes: u64,
}

impl Dfs {
    pub fn new(n: usize) -> Self {
        assert!((1..=30).contains(&n), "n must be in 1..=30");
        Self {
            n,
            full: (1u64 << n) - 1,
            lb: lower_bounds(n, STRIP_HEIGHT).anchored,
            budget: 0,
            rows: Vec::with_capacity(n),
            nodes: 0,
        }
    }

    /// 探索ノード数（rec を呼んだ回数、全予算の合計）
    pub fn nodes(&self) -> u64 {
        self.nodes
    }

    /// 予算を下界から 1 つずつ上げ、最初に塗れた予算で解を 1 つ返す
    pub fn solve(&mut self) -> Board {
        for budget in self.lb[self.n].. {
            self.budget = budget;
            if self.search() {
                let mut board = Board::new(self.n);
                for (r, &row) in self.rows.iter().enumerate() {
                    for c in 0..self.n {
                        board.painted[r * self.n + c] = row >> c & 1 == 1;
                    }
                }
                return board;
            }
        }
        unreachable!("painting every cell always satisfies the condition")
    }

    /// 今の予算で塗れるか。塗れたら self.rows に解が残る
    fn search(&mut self) -> bool {
        self.rows.clear();
        for first in 0..=self.full {
            // 解を左右反転しても解なので、行 0 が左右反転以下の解だけ探せばよい
            if first > mirror(self.n, first) {
                continue;
            }
            let c = first.count_ones() as usize;
            if self.admit(0, 0, first, c) {
                self.rows.push(first);
                if self.rec(0, 0, first, c) {
                    return true;
                }
                self.rows.pop();
            }
        }
        false
    }

    /// 行 k までの塗り数が c の状態 (up, cur) に進んでよいか（dp::forward の 1 行先読みと同じ）。
    /// 行 cur の条件を満たせない状態と、次の行の強制マスを足すと予算を超える状態は進まない。
    /// 最後の行なら、下端につながる（強制マスがない）ことも要求する
    fn admit(&self, k: usize, up: u64, cur: u64, c: usize) -> bool {
        let Some(forced) = forced_below(self.full, up, cur) else {
            return false;
        };
        let f = forced.count_ones() as usize;
        let rest = self.n - 1 - k;
        if rest == 0 {
            forced == 0 && c <= self.budget
        } else {
            c + self.lb[rest].max(f + self.lb[rest - 1]) <= self.budget
        }
    }

    /// 行 0..=k が決まっていて（self.rows）、admit を通った状態 (up, cur) から先を探す
    fn rec(&mut self, k: usize, up: u64, cur: u64, c: usize) -> bool {
        self.nodes += 1;
        if k + 1 == self.n {
            return true;
        }
        let forced = forced_below(self.full, up, cur).expect("admit checked this state");
        // cur の部分集合を全て列挙する
        let mut sub = cur;
        loop {
            let down = forced | sub;
            let w = c + down.count_ones() as usize;
            if self.admit(k + 1, cur, down, w) {
                self.rows.push(down);
                if self.rec(k + 1, cur, down, w) {
                    return true;
                }
                self.rows.pop();
            }
            if sub == 0 {
                break;
            }
            sub = (sub - 1) & cur;
        }
        false
    }
}

/// 列を 1 本おきに全部塗る自明な解の個数。
/// 列 0, 2, 4, ... を塗り、n が偶数なら最後の列も塗る。
/// 塗っていない列の各マスは左右が塗られ、上下は塗られていないので条件を満たす
pub fn upper_bound(n: usize) -> usize {
    n * (n / 2 + 1)
}

pub fn solve(n: usize) -> Board {
    Dfs::new(n).solve()
}

/// 最小解と探索ノード数を返す
pub fn solve_with_nodes(n: usize) -> (Board, u64) {
    let mut dfs = Dfs::new(n);
    let board = dfs.solve();
    (board, dfs.nodes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{brute, known};

    #[test]
    fn upper_bound_is_achievable() {
        for n in 1..=10 {
            let mut b = Board::new(n);
            for r in 0..n {
                for c in 0..n {
                    b.painted[r * n + c] = c % 2 == 0 || c == n - 1;
                }
            }
            assert!(b.is_valid(), "n = {n}");
            assert_eq!(b.count(), upper_bound(n), "n = {n}");
        }
    }

    #[test]
    fn matches_brute() {
        for n in 1..=4 {
            assert_eq!(solve(n).count(), brute::solve(n).count(), "n = {n}");
        }
    }

    #[test]
    fn matches_known() {
        for n in 1..=15 {
            let b = solve(n);
            assert!(b.is_valid(), "n = {n}");
            assert_eq!(Some(b.count()), known(n), "n = {n}");
        }
    }
}
