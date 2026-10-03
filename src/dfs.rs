//! 行単位の深さ優先探索（分枝限定法）
//!
//! 各行をビット列で表す。行 r とその上の行 r-1 が決まると、
//! 行 r の塗っていないマスが必要とする「下の塗りマス」の数は
//! 2 - (上 + 左 + 右) に確定する。これが 0 か 1 でなければ枝刈り。
//! したがって行 r+1 は、行 r の塗っていないマスの真下が強制され、
//! 自由に選べるのは行 r の塗ったマスの真下だけになる。

use crate::Board;

pub struct Dfs {
    n: usize,
    full: u64,
    rows: Vec<u64>,
    best: usize,
    best_rows: Vec<u64>,
    nodes: u64,
}

impl Dfs {
    pub fn new(n: usize) -> Self {
        assert!((1..64).contains(&n), "n must be in 1..64");
        let full = (1u64 << n) - 1;
        Self {
            n,
            full,
            rows: Vec::with_capacity(n),
            // 自明な解（upper_bound）も見つけられるよう、それより 1 多い値から始める
            best: upper_bound(n) + 1,
            best_rows: Vec::new(),
            nodes: 0,
        }
    }

    /// 探索ノード数（search を呼んだ回数）
    pub fn nodes(&self) -> u64 {
        self.nodes
    }

    /// 最小解を 1 つ返す
    pub fn solve(&mut self) -> Board {
        for first in 0..=self.full {
            let count = first.count_ones() as usize;
            if count >= self.best {
                continue;
            }
            self.rows.push(first);
            self.search(0, count);
            self.rows.pop();
        }
        let mut board = Board::new(self.n);
        for (r, &row) in self.best_rows.iter().enumerate() {
            for c in 0..self.n {
                board.painted[r * self.n + c] = row >> c & 1 == 1;
            }
        }
        board
    }

    /// 行 0..=r が決まっている。行 r+1 を選ぶ
    fn search(&mut self, r: usize, painted: usize) {
        self.nodes += 1;
        let up = if r == 0 { 0 } else { self.rows[r - 1] };
        let cur = self.rows[r];
        let Some(forced) = self.forced_below(up, cur) else {
            return;
        };

        if r + 1 == self.n {
            // 盤外の行は塗れない
            if forced == 0 && painted < self.best {
                self.best = painted;
                self.best_rows = self.rows.clone();
            }
            return;
        }

        let base = painted + forced.count_ones() as usize;
        if base >= self.best {
            return;
        }

        // cur の部分集合を全て列挙する
        let mut sub = cur;
        loop {
            let down = forced | sub;
            let next = base + sub.count_ones() as usize;
            if next < self.best {
                self.rows.push(down);
                self.search(r + 1, next);
                self.rows.pop();
            }
            if sub == 0 {
                break;
            }
            sub = (sub - 1) & cur;
        }
    }

    /// 行 cur の塗っていないマスが条件を満たすために、
    /// 真下に塗りが必要なマスのビット列。満たせないマスがあれば None
    fn forced_below(&self, up: u64, cur: u64) -> Option<u64> {
        let a = up;
        let b = (cur << 1) & self.full;
        let c = cur >> 1;
        let all3 = a & b & c;
        let ones = (a ^ b ^ c) & !all3;
        let twos = ((a & b) | (b & c) | (a & c)) & !all3;
        let unpainted = !cur & self.full;
        if unpainted & !(ones | twos) != 0 {
            return None;
        }
        Some(unpainted & ones)
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
        for n in 1..=7 {
            let b = solve(n);
            assert!(b.is_valid(), "n = {n}");
            assert_eq!(Some(b.count()), known(n), "n = {n}");
        }
    }
}
