//! 帯の最小塗り数を DFS で求める（bound::strip_min と同じ値。DP を使わない）
//!
//! 高さ h・幅 n の帯を、左から 1 列ずつ決める。帯の上の外側は自由（必要なら 1 個まで補える）、
//! 下の外側は bot_free なら自由、そうでなければ盤の外。左右は盤の端。
//!
//! 予算を 1 つずつ上げ、初めて塗れた予算を最小とする。枝刈りには、残りの列の最小の見積もりを使う。
//! 残りの列は「左の外側を自由とみなした、同じ高さの細い帯」と見なせるので、その最小 lf[w] を
//! 幅 w の小さい順に同じ DFS で求めておき、見積もりに使う（DP は使わない）。
//! lf は幅について増加する（幅 w の解から左端の列を除くと、幅 w-1 の解になる）ので、
//! 幅 w の探索は予算 lf[w-1] から始められる。

/// 列 c の白マスが条件を満たすために、右の列が満たすべき条件 (必ず塗るマス, 塗ってはいけないマス)。
/// 帯の上の外側は自由。left_free なら列 c の左隣も自由（帯の左端で、左の外側を自由とみなすとき）。
/// 満たせない白マスがあれば None
fn requirements(
    h: usize,
    l: usize,
    c: usize,
    bot_free: bool,
    left_free: bool,
) -> Option<(usize, usize)> {
    let (mut must1, mut must0) = (0, 0);
    for i in 0..h {
        if c >> i & 1 == 1 {
            continue;
        }
        // s: 決まっている塗りの数、f: 好きに塗れる外側の隣の数
        let mut s = l >> i & 1;
        let mut f = usize::from(left_free);
        if i > 0 {
            s += c >> (i - 1) & 1;
        } else {
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

struct Search {
    h: usize,
    mask: usize,
    bot_free: bool,
    /// lf[w]: 左の外側を自由とみなした、幅 w の帯の最小（求めた分だけ）
    lf: Vec<usize>,
    budget: usize,
    /// 探索ノード数（rec を呼んだ回数）
    nodes: u64,
}

impl Search {
    /// 幅 w の帯を予算以内で塗れるか。left_free なら帯の左の外側を自由とみなす
    fn feasible(&mut self, w: usize, left_free: bool) -> bool {
        for c0 in 0..=self.mask {
            let cost = c0.count_ones() as usize;
            if cost + self.lf[w - 1] > self.budget {
                continue;
            }
            if self.rec(w, 0, 0, c0, cost, left_free) {
                return true;
            }
        }
        false
    }

    /// 列 0..=j が決まっていて、列 j-1, j が (l, c)、ここまでの塗り数が cost
    fn rec(
        &mut self,
        w: usize,
        j: usize,
        l: usize,
        c: usize,
        cost: usize,
        left_free: bool,
    ) -> bool {
        self.nodes += 1;
        let Some((must1, must0)) = requirements(self.h, l, c, self.bot_free, j == 0 && left_free)
        else {
            return false;
        };
        if j + 1 == w {
            // 右の外側は盤の外なので塗れない
            return must1 == 0 && cost <= self.budget;
        }
        // 次の列 r の右に残る列の数
        let rem = w - 2 - j;
        let free = self.mask & !must1 & !must0;
        let mut sub = free;
        loop {
            let r = must1 | sub;
            let c2 = cost + r.count_ones() as usize;
            // 残りの列の見積もり。r の次の列の強制マスまで先読みする
            let bound = if rem == 0 {
                Some(c2)
            } else {
                requirements(self.h, c, r, self.bot_free, false).map(|(m1, _)| {
                    let next = m1.count_ones() as usize;
                    c2 + self.lf[rem].max(next + self.lf[rem - 1])
                })
            };
            if bound.is_some_and(|b| b <= self.budget) && self.rec(w, j + 1, c, r, c2, left_free) {
                return true;
            }
            if sub == 0 {
                break;
            }
            sub = (sub - 1) & free;
        }
        false
    }

    /// 幅 w の帯の最小。予算を start から 1 つずつ上げる
    fn minimum(&mut self, w: usize, left_free: bool, start: usize) -> usize {
        let mut budget = start;
        loop {
            self.budget = budget;
            if self.feasible(w, left_free) {
                return budget;
            }
            budget += 1;
        }
    }
}

/// 高さ h・幅 n の帯の最小塗り数（上の外側は自由、bot_free なら下の外側も自由）と、探索ノード数。
/// bound::strip_min(n, h, true, bot_free) と同じ値
pub fn strip_min_dfs(n: usize, h: usize, bot_free: bool) -> (usize, u64) {
    assert!(n >= 1 && (1..=30).contains(&h));
    let mut s = Search {
        h,
        mask: (1usize << h) - 1,
        bot_free,
        lf: vec![0],
        budget: 0,
        nodes: 0,
    };
    // 左の外側を自由とみなした細い帯の最小を、幅の小さい順に求める
    for w in 1..n {
        let start = s.lf[w - 1];
        let v = s.minimum(w, true, start);
        s.lf.push(v);
    }
    // 左右とも盤の端の、幅 n の帯
    let start = s.lf[n - 1];
    let v = s.minimum(n, false, start);
    (v, s.nodes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bound::strip_min;

    #[test]
    fn matches_dp() {
        for n in 1..=14 {
            for h in 1..=n.min(7) {
                for bot_free in [true, false] {
                    assert_eq!(
                        strip_min_dfs(n, h, bot_free).0,
                        strip_min(n, h, true, bot_free),
                        "n = {n}, h = {h}, bot_free = {bot_free}"
                    );
                }
            }
        }
    }
}
