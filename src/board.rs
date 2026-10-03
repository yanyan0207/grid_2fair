use std::fmt;

/// n×n の盤面。painted[r * n + c] が true なら塗ったマス
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Board {
    pub n: usize,
    pub painted: Vec<bool>,
}

impl Board {
    pub fn new(n: usize) -> Self {
        Self {
            n,
            painted: vec![false; n * n],
        }
    }

    /// 下位ビットから行優先で並べたビット列から作る（n*n <= 64）
    pub fn from_mask(n: usize, mask: u64) -> Self {
        let painted = (0..n * n).map(|i| mask >> i & 1 == 1).collect();
        Self { n, painted }
    }

    pub fn get(&self, r: usize, c: usize) -> bool {
        self.painted[r * self.n + c]
    }

    pub fn count(&self) -> usize {
        self.painted.iter().filter(|&&p| p).count()
    }

    /// (r, c) の上下左右にある塗ったマスの数
    pub fn painted_neighbors(&self, r: usize, c: usize) -> usize {
        let n = self.n;
        let mut k = 0;
        if r > 0 && self.get(r - 1, c) {
            k += 1;
        }
        if r + 1 < n && self.get(r + 1, c) {
            k += 1;
        }
        if c > 0 && self.get(r, c - 1) {
            k += 1;
        }
        if c + 1 < n && self.get(r, c + 1) {
            k += 1;
        }
        k
    }

    /// 塗っていない全マスが、ちょうど 2 個の塗ったマスに隣接しているか
    pub fn is_valid(&self) -> bool {
        (0..self.n)
            .all(|r| (0..self.n).all(|c| self.get(r, c) || self.painted_neighbors(r, c) == 2))
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for r in 0..self.n {
            let line: String = (0..self.n)
                .map(|c| if self.get(r, c) { '#' } else { '.' })
                .collect();
            writeln!(f, "{line}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_is_invalid() {
        assert!(!Board::new(2).is_valid());
    }

    #[test]
    fn diagonal_2x2_is_valid() {
        // #.
        // .#
        let b = Board::from_mask(2, 0b1001);
        assert!(b.is_valid());
        assert_eq!(b.count(), 2);
    }
}
