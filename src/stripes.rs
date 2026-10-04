//! 縞模様の行
//!
//! 最適解は、上下と左右の端を除くと、どの行も周期 3 の縞（c ≡ s (mod 3) の列だけを塗る）になっている。
//! stripes_dfs.rs は、端以外の行をこの形に制限して解を作る。

/// 既定の制限の幅（自由にする上下の行数と左右の列数）。
/// n = 12〜22 の最適解は、上下 2 行と左右 2 列を除くと全部の行が縞になっていた
pub const DEFAULT_MARGIN: usize = 2;

/// 幅 n の行のうち、列 margin..n-margin の部分が周期 3 の縞
/// （c ≡ s (mod 3) の列だけを塗る、s = 0, 1, 2 のどれか）と一致するもの。
/// 左右の端の margin 列は何でもよい。左右反転しても縞の行は縞の行
#[derive(Clone, Debug)]
pub struct Stripes {
    n: usize,
    margin: usize,
    /// 縞を要求する列のビット
    interior: u64,
    /// 縞の 3 通りのずれ方それぞれの、interior の中で塗るマス
    masks: [u64; 3],
}

impl Stripes {
    /// 左右 margin 列を自由にする。内側が空なら None
    pub fn new(n: usize, margin: usize) -> Option<Self> {
        if 2 * margin >= n {
            return None;
        }
        let cols = margin..n - margin;
        let interior = cols.clone().fold(0u64, |m, c| m | 1 << c);
        let masks = [0, 1, 2].map(|s| {
            cols.clone()
                .filter(|c| c % 3 == s)
                .fold(0u64, |m, c| m | 1 << c)
        });
        Some(Self {
            n,
            margin,
            interior,
            masks,
        })
    }

    /// 自由にする上下の行数と左右の列数
    pub fn margin(&self) -> usize {
        self.margin
    }

    /// row が縞の行か
    pub fn is_striped(&self, row: u64) -> bool {
        self.masks.contains(&(row & self.interior))
    }

    /// 縞の行を全部（縞のずれ方 3 通り × 左右の端の塗り方 2^(2 margin) 通り）
    pub fn rows(&self) -> Vec<u64> {
        let full = (1u64 << self.n) - 1;
        let edge = full & !self.interior;
        let mut masks = self.masks.to_vec();
        masks.sort_unstable();
        masks.dedup();
        let mut rows = Vec::new();
        for mask in masks {
            // edge の部分集合を全て列挙する
            let mut e = edge;
            loop {
                rows.push(mask | e);
                if e == 0 {
                    break;
                }
                e = (e - 1) & edge;
            }
        }
        rows
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::row::mirror;

    #[test]
    fn striped_rows() {
        let s = Stripes::new(10, 2).unwrap();
        // 列 2..=7 が縞なら、端の列は何でもよい
        assert!(s.is_striped(0b11_0100_1001)); // 列 0, 3, 6 と 8, 9
        assert!(!s.is_striped(0b00_0001_1000)); // 列 3, 4 は縞ではない
        let rows = s.rows();
        assert_eq!(rows.len(), 3 * 16);
        assert!(rows.iter().all(|&r| s.is_striped(r)));
    }

    #[test]
    fn mirror_keeps_stripes() {
        for n in 5..=20 {
            let s = Stripes::new(n, 2).unwrap();
            for row in s.rows() {
                assert!(s.is_striped(mirror(n, row)), "n = {n}, row = {row:b}");
            }
        }
    }

    #[test]
    fn wide_margin_means_none() {
        assert!(Stripes::new(4, 2).is_none());
        assert!(Stripes::new(5, 2).is_some());
    }
}
