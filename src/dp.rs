//! 予算付きの行単位 DP（下界による枝刈り）
//!
//! 状態は「直前の 2 行」(up, cur)。行 r から先の最小値はこの 2 行だけで決まるので、
//! 上から 1 行ずつ、到達できる状態とそこまでの最小塗り数を更新していく。
//!
//! 予算 budget を決めて「budget 個以下で塗れるか」を判定する。
//! 行 0..=k の塗り数 c に、残りの行に最低限必要な塗り数 anchored[n-1-k] を足して
//! 予算を超える状態は捨てる。下界は高さ h の帯の最小値（strip_min）から作る。
//!
//! 予算を下界から 1 つずつ上げ、最初に解が見つかった予算が a(n)。
//! それより小さい予算では「解なし」が示されたことになる。
//!
//! 判定には直前の行の表しか要らないので、表は 2 枚だけ持つ。
//! そのため盤面はたどれない。盤面は、確定した端の 2 行から出発して
//! 反対側の端の 2 行を確定させる DP を、向きを交互に変えて繰り返して復元する（reconstruct）。

use rustc_hash::FxHashMap as HashMap;

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

/// 1 回の予算判定の結果
pub struct Outcome {
    /// 予算以内で塗れるときの最小塗り数と、そのときの最後の状態。塗れなければ None
    pub min: Option<(usize, u64)>,
    /// 遷移の数
    pub transitions: u64,
    /// 1 行あたりの状態数の最大
    pub max_states: usize,
}

/// 状態のキー（上の行, 今の行）
fn key(up: u64, cur: u64) -> u64 {
    up << 32 | cur
}

fn split(st: u64) -> (u64, u64) {
    (st >> 32, st & 0xFFFF_FFFF)
}

/// 幅 n の行を左右反転する
fn mirror(n: usize, row: u64) -> u64 {
    row.reverse_bits() >> (64 - n)
}

/// 状態とその左右反転のうち、キーが小さい方。
/// 左右反転した状態は、その先の探索も鏡写しで最小塗り数も同じなので、1 つにまとめてよい
fn canonical(n: usize, up: u64, cur: u64) -> u64 {
    key(up, cur).min(key(mirror(n, up), mirror(n, cur)))
}

/// 行 first の状態 start から行 last まで DP を進める。
/// keep(k, c): 行 k までの塗り数が c の状態を残すか。
/// 行 last の各状態について on_last(上の行, 今の行, 塗り数) を呼ぶ。
/// 次の行の表は今の行の表だけから作れるので、表は 2 枚だけ持つ。
/// symmetric なら、左右反転で重なる状態を 1 つにまとめる（問題全体が左右対称なときだけ使える）。
/// 戻り値は (遷移の数, 1 行あたりの状態数の最大)
fn forward(
    n: usize,
    start: impl IntoIterator<Item = (u64, u64, usize)>,
    first: usize,
    last: usize,
    symmetric: bool,
    keep: impl Fn(usize, usize) -> bool,
    mut on_last: impl FnMut(u64, u64, usize),
) -> (u64, usize) {
    assert!((1..=30).contains(&n), "n must be in 1..=30");
    let full = (1u64 << n) - 1;
    let make_key = |up, cur| {
        if symmetric {
            canonical(n, up, cur)
        } else {
            key(up, cur)
        }
    };
    // 状態 → 行 k までの最小塗り数
    let mut table: HashMap<u64, u16> = HashMap::default();
    for (up, cur, c) in start {
        if keep(first, c) {
            let e = table.entry(make_key(up, cur)).or_insert(u16::MAX);
            *e = (*e).min(c as u16);
        }
    }
    let mut transitions = 0u64;
    let mut max_states = table.len();
    for k in first..last {
        let mut next: HashMap<u64, u16> = HashMap::default();
        for (&st, &v) in &table {
            let (up, cur) = split(st);
            let Some(forced) = forced_below(full, up, cur) else {
                continue;
            };
            let mut sub = cur;
            loop {
                let down = forced | sub;
                let w = v + down.count_ones() as u16;
                transitions += 1;
                if keep(k + 1, w as usize) {
                    let e = next.entry(make_key(cur, down)).or_insert(u16::MAX);
                    if w < *e {
                        *e = w;
                    }
                }
                if sub == 0 {
                    break;
                }
                sub = (sub - 1) & cur;
            }
        }
        // 行 k の表はもう使わないので捨てる
        table = next;
        max_states = max_states.max(table.len());
    }
    for (&st, &v) in &table {
        let (up, cur) = split(st);
        on_last(up, cur, v as usize);
    }
    (transitions, max_states)
}

/// budget 個以下で塗れるかを判定する
pub fn search(n: usize, budget: usize, bounds: &Bounds) -> Outcome {
    // 盤も条件も左右対称なので、左右反転で重なる状態をまとめる。
    // 最後の状態が鏡写しになっていても、解を左右反転すればその最後の 2 行を持つ解になる
    search_impl(n, budget, bounds, true)
}

/// search の本体。symmetric を切れるようにしてあるのは、テストでまとめない場合と比べるため
fn search_impl(n: usize, budget: usize, bounds: &Bounds, symmetric: bool) -> Outcome {
    let full = (1u64 << n) - 1;
    let lb = &bounds.anchored;
    let mut min: Option<(usize, u64)> = None;
    let (transitions, max_states) = forward(
        n,
        (0..=full).map(|row| (0, row, row.count_ones() as usize)),
        0,
        n - 1,
        symmetric,
        |k, c| c + lb[n - 1 - k] <= budget,
        |up, cur, c| {
            // 盤外の行は塗れないので、最後の行に強制マスがあってはいけない
            if forced_below(full, up, cur) == Some(0) && min.is_none_or(|(m, _)| c < m) {
                min = Some((c, key(up, cur)));
            }
        },
    );
    Outcome {
        min,
        transitions,
        max_states,
    }
}

/// 行 x, y の下に行 z が正しくつながるか（行 y の条件が満たされるか）
fn connects(full: u64, x: u64, y: u64, z: u64) -> bool {
    let Some(forced) = forced_below(full, x, y) else {
        return false;
    };
    // 行 z は forced を含み、残りは行 y の塗ったマスの真下だけ
    z & forced == forced && z & !forced & !y == 0
}

/// 両端の行が決まっているとき、間の行を DP で探し、
/// 遠い側の端に接する 2 行（未定が 1 行なら 1 行）を確定させる。
/// near: 出発側の端から順に並べた確定済みの行（1 行以上）。
/// far: 反対側の端から順に並べた確定済みの行。
/// 盤は上下反転しても同じ問題なので、向きはどちらでもよい。
/// 戻り値は (far の続きに足す行, 遷移の数, 状態数の最大)
fn pin_far_end(
    n: usize,
    total: usize,
    near: &[u64],
    far: &[u64],
    bounds: &Bounds,
) -> (Vec<u64>, u64, usize) {
    let full = (1u64 << n) - 1;
    let (t, s) = (near.len(), far.len());
    assert!(t >= 1 && t + s < n);
    // 未定の行は t..=b
    let b = n - 1 - s;
    let near_cost: usize = near.iter().map(|x| x.count_ones() as usize).sum();
    let far_cost: usize = far.iter().map(|x| x.count_ones() as usize).sum();
    let start_up = if t >= 2 { near[t - 2] } else { 0 };
    let (lb, free) = (&bounds.anchored, &bounds.free);
    let mut found = None;
    let (transitions, max_states) = forward(
        n,
        [(start_up, near[t - 1], near_cost)],
        t - 1,
        b,
        // 両端の行が固定されていて左右対称ではないので、まとめない
        false,
        |k, c| {
            if k < b {
                // 行 k+1..=b は未定、far 側は確定
                c + lb[n - 1 - k].max(free[b - k] + far_cost) <= total
            } else {
                c + far_cost <= total
            }
        },
        |x, y, c| {
            if found.is_some() || c + far_cost != total {
                return;
            }
            let ok = match s {
                0 => forced_below(full, x, y) == Some(0),
                1 => connects(full, x, y, far[0]) && forced_below(full, y, far[0]) == Some(0),
                _ => connects(full, x, y, far[s - 1]) && connects(full, y, far[s - 1], far[s - 2]),
            };
            if ok {
                found = Some((x, y));
            }
        },
    );
    let (x, y) = found.expect("rows taken from a minimum solution always have a connecting middle");
    // 行 b は y、行 b-1 は x。x が near 側の確定済みの行なら y だけ
    let rows = if b > t { vec![y, x] } else { vec![y] };
    (rows, transitions, max_states)
}

/// 塗り数 total の解の最後の状態 last から、盤面を復元する。
/// 表を 2 枚しか持たないので、1 回の DP で確定できるのは遠い側の端の 2 行だけ。
/// 確定した端から出発し、反対側の端の 2 行を確定させる DP を、向きを交互に変えて繰り返す。
/// 出発点が 1 状態に固定されるので、どの回も最初の探索より小さく収まる。
/// on_pass(確定した行数, 遷移の数, 状態数の最大) は 1 回ごとに呼ばれる
pub fn reconstruct(
    n: usize,
    total: usize,
    last: u64,
    bounds: &Bounds,
    mut on_pass: impl FnMut(usize, u64, usize),
) -> Board {
    // top: 上端から順に並べた確定済みの行、bottom: 下端から順に並べた確定済みの行
    let mut top: Vec<u64> = Vec::new();
    let (up, cur) = split(last);
    let mut bottom = vec![cur];
    if n >= 2 {
        bottom.push(up);
    }
    let mut from_bottom = true;
    while top.len() + bottom.len() < n {
        if from_bottom {
            let (rows, transitions, max_states) = pin_far_end(n, total, &bottom, &top, bounds);
            top.extend(rows);
            on_pass(top.len() + bottom.len(), transitions, max_states);
        } else {
            let (rows, transitions, max_states) = pin_far_end(n, total, &top, &bottom, bounds);
            bottom.extend(rows);
            on_pass(top.len() + bottom.len(), transitions, max_states);
        }
        from_bottom = !from_bottom;
    }
    let rows: Vec<u64> = top.into_iter().chain(bottom.into_iter().rev()).collect();
    let mut board = Board::new(n);
    for (r, &row) in rows.iter().enumerate() {
        for c in 0..n {
            board.painted[r * n + c] = row >> c & 1 == 1;
        }
    }
    board
}

/// 予算を下界から 1 つずつ上げ、最初に解ありとなった予算で解を 1 つ返す。
/// on_budget は各予算の判定結果ごとに、on_pass は復元の DP 1 回ごとに呼ばれる
pub fn solve_by_budget(
    n: usize,
    mut on_budget: impl FnMut(usize, &Outcome),
    on_pass: impl FnMut(usize, u64, usize),
) -> Board {
    let bounds = lower_bounds(n, STRIP_HEIGHT);
    for budget in bounds.anchored[n].. {
        let outcome = search(n, budget, &bounds);
        on_budget(budget, &outcome);
        if let Some((total, last)) = outcome.min {
            return reconstruct(n, total, last, &bounds, on_pass);
        }
    }
    unreachable!()
}

pub fn solve(n: usize) -> Board {
    solve_by_budget(n, |_, _| {}, |_, _, _| {})
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
            assert!(lb.anchored[n] <= known(n).unwrap(), "n = {n}");
        }
    }

    #[test]
    fn matches_dfs() {
        for n in 1..=7 {
            assert_eq!(solve(n).count(), dfs::solve(n).count(), "n = {n}");
        }
    }

    #[test]
    fn mirror_roundtrip() {
        for n in 1..=30 {
            let full = (1u64 << n) - 1;
            for row in [0, 1, full, full >> 1, 0b1011 & full] {
                assert_eq!(mirror(n, mirror(n, row)), row, "n = {n}, row = {row:b}");
                assert_eq!(
                    mirror(n, row).count_ones(),
                    row.count_ones(),
                    "n = {n}, row = {row:b}"
                );
            }
            // 左端の 1 ビットは右端に移る
            assert_eq!(mirror(n, 1), 1 << (n - 1), "n = {n}");
        }
    }

    /// 左右反転で重なる状態をまとめても、まとめない場合と判定結果が変わらない
    #[test]
    fn symmetric_matches_plain() {
        for n in 1..=12 {
            let lb = lower_bounds(n, STRIP_HEIGHT);
            let a = known(n).unwrap();
            for budget in lb.anchored[n]..=a {
                let sym = search_impl(n, budget, &lb, true);
                let plain = search_impl(n, budget, &lb, false);
                assert_eq!(
                    sym.min.map(|(c, _)| c),
                    plain.min.map(|(c, _)| c),
                    "n = {n}, budget = {budget}"
                );
            }
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
            assert!(search(n, a - 1, &lb).min.is_none(), "n = {n}");
        }
    }
}
