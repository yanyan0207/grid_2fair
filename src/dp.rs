//! 予算付きの行単位 DP（下界による枝刈り）
//!
//! 状態は (cur, forced)。cur は今の行、forced は cur の真下で必ず塗るマス（row::forced_below）。
//! 次の行の候補は「forced ∪ cur の塗ったマスの真下の部分集合」で、その行が条件を満たせるかも
//! cur と次の行だけで決まる。なので行 r から先の最小値は (cur, forced) だけで決まり、上の行は要らない。
//! 上の行が違っても forced が同じなら 1 つの状態にまとまる。
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
//! そのため盤面はたどれない。盤面は、確定した端の行から出発して
//! 反対側の端の 1 行を確定させる DP を、向きを交互に変えて繰り返して復元する（reconstruct）。

use rustc_hash::FxHashMap as HashMap;

use crate::Board;
use crate::bound::{Bounds, STRIP_HEIGHT, lower_bounds};
use crate::row::{forced_below, mirror};

/// 1 回の予算判定の結果
pub struct Outcome {
    /// 予算以内で塗れるときの最小塗り数と、そのときの最後の行。塗れなければ None
    pub min: Option<(usize, u64)>,
    /// 遷移の数
    pub transitions: u64,
    /// 1 行あたりの状態数（(cur, forced) の組の数）の最大
    pub max_states: usize,
}

/// 1 行分の表を作ったときの集計（計測用）
#[derive(Clone, Debug, Default)]
pub struct RowStats {
    /// 試した遷移の数（最初の行では、候補の数）
    pub transitions: u64,
    /// 行き止まり（行 cur の条件を満たせない）で表に入れなかった数
    pub dead: u64,
    /// 予算超過で表に入れなかった数
    pub over: u64,
    /// 表にすでにある状態（左右反転を含む）に重なった数
    pub merged: u64,
}

/// 表の 1 行分。今の行 cur → その行の状態の配列 [(cur の真下の強制マス forced, その行までの最小塗り数)]。
/// 同じ cur を持つ状態は cur を 1 回だけ保存し、forced と塗り数だけを並べて持つ。
/// 左右対称でまとめるときは、(cur, forced) と左右反転した組のうち小さい方で入れる（canonical_state）
pub type Table = HashMap<u64, Vec<(u32, u16)>>;

/// 状態 (cur, forced) と、両方を左右反転した組のうち小さい方。
/// 左右反転した状態は、その先の探索も鏡写しで最小塗り数も同じなので、1 つにまとめてよい
fn canonical_state(n: usize, cur: u64, forced: u64) -> (u64, u64) {
    let mirrored = (mirror(n, cur), mirror(n, forced));
    mirrored.min((cur, forced))
}

/// 表の状態の数（(cur, forced) の組の数）
pub fn table_states(table: &Table) -> usize {
    table.values().map(Vec::len).sum()
}

/// 行 first の状態 start から行 last まで DP を進める。
/// start は (上の行, 今の行, 塗り数) で与え、forced はここで求める。
/// keep(k, c, forced): 行 k までの塗り数が c で、行 k+1 の強制マスが forced 個の状態を残すか。
/// 行 k の条件を満たせない状態（forced_below が None）は、keep を呼ばずに表に入れない。
/// 行 last の各状態について on_last(今の行, forced, 塗り数) を呼ぶ。
/// 行 k の表ができるたびに on_row(k, 表, 集計) を呼ぶ（計測用）。
/// 次の行の表は今の行の表だけから作れるので、表は 2 枚だけ持つ。
/// symmetric なら、左右反転で重なる状態を 1 つにまとめる（問題全体が左右対称なときだけ使える）。
/// 戻り値は (遷移の数, 1 行あたりの状態数の最大)
#[allow(clippy::too_many_arguments)]
fn forward(
    n: usize,
    start: impl IntoIterator<Item = (u64, u64, usize)>,
    first: usize,
    last: usize,
    symmetric: bool,
    keep: impl Fn(usize, usize, usize) -> bool,
    mut on_last: impl FnMut(u64, u64, usize),
    on_row: &mut dyn FnMut(usize, &Table, &RowStats),
) -> (u64, usize) {
    assert!((1..=30).contains(&n), "n must be in 1..=30");
    let full = (1u64 << n) - 1;
    // 行 k までの塗り数が c の状態を、上の行 up と今の行 cur から作って表に入れる。
    // 先に cur の真下の強制マスを求め、行 cur の条件を満たせない状態と、
    // 強制マスを足すと予算を超える状態は入れない（1 行先読み）
    let admit = |table: &mut Table, stats: &mut RowStats, k: usize, up: u64, cur: u64, c: usize| {
        let Some(forced) = forced_below(full, up, cur) else {
            stats.dead += 1;
            return;
        };
        if !keep(k, c, forced.count_ones() as usize) {
            stats.over += 1;
            return;
        }
        let (cur, forced) = if symmetric {
            canonical_state(n, cur, forced)
        } else {
            (cur, forced)
        };
        let group = table.entry(cur).or_default();
        // 同じ (cur, forced) なら先の探索も同じなので、塗り数の小さい方だけ残す
        match group.iter_mut().find(|(f, _)| *f as u64 == forced) {
            Some(e) => {
                stats.merged += 1;
                e.1 = e.1.min(c as u16);
            }
            None => group.push((forced as u32, c as u16)),
        }
    };
    let mut table = Table::default();
    let mut stats = RowStats::default();
    for (up, cur, c) in start {
        stats.transitions += 1;
        admit(&mut table, &mut stats, first, up, cur, c);
    }
    on_row(first, &table, &stats);
    let mut transitions = 0u64;
    let mut max_states = table_states(&table);
    for k in first..last {
        let mut next = Table::default();
        let mut stats = RowStats::default();
        for (&cur, group) in &table {
            // 次の行は forced ∪ (cur の部分集合)。部分集合の列挙は同じ cur の状態で共有する
            let mut sub = cur;
            loop {
                for &(forced, v) in group {
                    let down = forced as u64 | sub;
                    stats.transitions += 1;
                    admit(
                        &mut next,
                        &mut stats,
                        k + 1,
                        cur,
                        down,
                        v as usize + down.count_ones() as usize,
                    );
                }
                if sub == 0 {
                    break;
                }
                sub = (sub - 1) & cur;
            }
        }
        // 行 k の表はもう使わないので捨てる
        table = next;
        transitions += stats.transitions;
        max_states = max_states.max(table_states(&table));
        on_row(k + 1, &table, &stats);
    }
    for (&cur, group) in &table {
        for &(forced, v) in group {
            on_last(cur, forced as u64, v as usize);
        }
    }
    (transitions, max_states)
}

/// 判定の DP で、行 k までの塗り数が c、行 k+1 の強制マスが forced 個の状態から先の、
/// 最終的な塗り数の下界。行 k+1 が盤外（k が最後の行）で強制マスがあれば、解につながらないので None。
/// 予算以下なら表に残す。lb は Bounds::anchored
pub fn search_need(n: usize, lb: &[usize], k: usize, c: usize, forced: usize) -> Option<usize> {
    // 残りは n-1-k 行。forced は行 k+1 で必ず塗るので、その先の行の下界に足せる
    let rest = n - 1 - k;
    if rest == 0 {
        (forced == 0).then_some(c)
    } else {
        Some(c + lb[rest].max(forced + lb[rest - 1]))
    }
}

/// budget 個以下で塗れるかを判定する
pub fn search(n: usize, budget: usize, bounds: &Bounds) -> Outcome {
    // 盤も条件も左右対称なので、左右反転で重なる状態をまとめる。
    // 最後の状態が鏡写しになっていても、解を左右反転すればその最後の 2 行を持つ解になる
    search_impl(n, budget, bounds, true, &mut |_, _, _| {})
}

/// search と同じ。行 k の表ができるたびに on_row(k, 表, 集計) を呼ぶ（計測用）。
pub fn search_profiled(
    n: usize,
    budget: usize,
    bounds: &Bounds,
    mut on_row: impl FnMut(usize, &Table, &RowStats),
) -> Outcome {
    search_impl(n, budget, bounds, true, &mut on_row)
}

/// search の本体。symmetric を切れるようにしてあるのは、テストでまとめない場合と比べるため
fn search_impl(
    n: usize,
    budget: usize,
    bounds: &Bounds,
    symmetric: bool,
    on_row: &mut dyn FnMut(usize, &Table, &RowStats),
) -> Outcome {
    let full = (1u64 << n) - 1;
    let lb = &bounds.anchored;
    let mut min: Option<(usize, u64)> = None;
    let (transitions, max_states) = forward(
        n,
        (0..=full).map(|row| (0, row, row.count_ones() as usize)),
        0,
        n - 1,
        symmetric,
        |k, c, forced| search_need(n, lb, k, c, forced).is_some_and(|need| need <= budget),
        |cur, forced, c| {
            // 盤外の行は塗れないので、最後の行に強制マスがあってはいけない
            if forced == 0 && min.is_none_or(|(m, _)| c < m) {
                min = Some((c, cur));
            }
        },
        on_row,
    );
    Outcome {
        min,
        transitions,
        max_states,
    }
}

/// 行 y の真下の強制マスが forced のとき、行 y の下に行 z が正しくつながるか（行 y の条件が満たされるか）。
/// 行 z は forced を含み、残りは行 y の塗ったマスの真下だけ
fn fits(forced: u64, y: u64, z: u64) -> bool {
    z & forced == forced && z & !forced & !y == 0
}

/// 行 x, y の下に行 z が正しくつながるか
fn connects(full: u64, x: u64, y: u64, z: u64) -> bool {
    forced_below(full, x, y).is_some_and(|forced| fits(forced, y, z))
}

/// 両端の行が決まっているとき、間の行を DP で探し、遠い側の端に接する 1 行を確定させる。
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
) -> (u64, u64, usize) {
    let full = (1u64 << n) - 1;
    let (t, s) = (near.len(), far.len());
    assert!(t >= 1 && t + s < n);
    // 未定の行は t..=b
    let b = n - 1 - s;
    let near_cost: usize = near.iter().map(|x| x.count_ones() as usize).sum();
    let far_cost: usize = far.iter().map(|x| x.count_ones() as usize).sum();
    // near が 1 行だけなら、その手前は盤の外
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
        |k, c, forced| {
            if k < b {
                // 行 k+1..=b は未定、far 側は確定。forced は未定の行 k+1 に入るので、
                // その先の行の下界に足せる
                let rest = n - 2 - k; // 行 k+2..=n-1 の行数
                let mid = b - k - 1; // 行 k+2..=b の行数
                let bound = lb[n - 1 - k]
                    .max(free[b - k] + far_cost)
                    .max(forced + lb[rest])
                    .max(forced + free[mid] + far_cost);
                c + bound <= total
            } else {
                // 未定の行はもうない。forced は far 側の確定済みの行か盤外に入り、
                // 確定済みの行の分は far_cost に含まれているので足さない
                c + far_cost <= total
            }
        },
        // 行 b の状態 (y, forced)。forced は y の真下（far 側）の強制マス
        |y, forced, c| {
            if found.is_some() || c + far_cost != total {
                return;
            }
            let ok = match s {
                0 => forced == 0,
                1 => fits(forced, y, far[0]) && forced_below(full, y, far[0]) == Some(0),
                _ => fits(forced, y, far[s - 1]) && connects(full, y, far[s - 1], far[s - 2]),
            };
            if ok {
                found = Some(y);
            }
        },
        &mut |_, _, _| {},
    );
    let y = found.expect("rows taken from a minimum solution always have a connecting middle");
    (y, transitions, max_states)
}

/// 塗り数 total の解の最後の行 last から、盤面を復元する。
/// 表を 2 枚しか持たず、状態に上の行も持たないので、1 回の DP で確定できるのは遠い側の端の 1 行だけ。
/// 確定した端から出発し、反対側の端の 1 行を確定させる DP を、向きを交互に変えて繰り返す。
/// 最初は最後の行だけから出発する（その先は盤の外なので、上の行は要らない）。
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
    let mut bottom = vec![last];
    let mut from_bottom = true;
    while top.len() + bottom.len() < n {
        if from_bottom {
            let (row, transitions, max_states) = pin_far_end(n, total, &bottom, &top, bounds);
            top.push(row);
            on_pass(top.len() + bottom.len(), transitions, max_states);
        } else {
            let (row, transitions, max_states) = pin_far_end(n, total, &top, &bottom, bounds);
            bottom.push(row);
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
    fn matches_dfs() {
        for n in 1..=15 {
            assert_eq!(solve(n).count(), dfs::solve(n).count(), "n = {n}");
        }
    }

    /// 左右反転で重なる状態をまとめても、まとめない場合と判定結果が変わらない
    #[test]
    fn symmetric_matches_plain() {
        for n in 1..=12 {
            let lb = lower_bounds(n, STRIP_HEIGHT);
            let a = known(n).unwrap();
            for budget in lb.anchored[n]..=a {
                let sym = search_impl(n, budget, &lb, true, &mut |_, _, _| {});
                let plain = search_impl(n, budget, &lb, false, &mut |_, _, _| {});
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
        for n in 1..=15 {
            let b = solve(n);
            assert!(b.is_valid(), "n = {n}");
            assert_eq!(Some(b.count()), known(n), "n = {n}");
        }
    }

    #[test]
    fn budget_below_answer_is_infeasible() {
        for n in 1..=15 {
            let lb = lower_bounds(n, STRIP_HEIGHT);
            let a = known(n).unwrap();
            assert!(search(n, a - 1, &lb).min.is_none(), "n = {n}");
        }
    }
}
