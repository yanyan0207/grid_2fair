//! OEIS A344719
//!
//! n×n 格子のいくつかのマスを塗り、塗っていない全てのマスが
//! 上下左右でちょうど 2 個の塗ったマスに隣接するようにする。
//! このとき塗るマスの最小個数を a(n) とする。

pub mod board;
pub mod brute;
pub mod dfs;

pub use board::Board;

/// OEIS に載っている既知の値 a(1)..=a(15)
pub const KNOWN: [usize; 15] = [1, 2, 5, 8, 11, 17, 21, 28, 35, 42, 51, 60, 69, 80, 91];

/// 既知の a(n)。範囲外なら None
pub fn known(n: usize) -> Option<usize> {
    n.checked_sub(1).and_then(|i| KNOWN.get(i).copied())
}
