use std::time::Instant;

use clap::{Parser, ValueEnum};
use grid_2fair::{brute, dfs, dp, known};

/// 使う探索アルゴリズム
#[derive(Clone, Copy, Debug, ValueEnum)]
enum Algo {
    /// 全パターンを調べる総当たり（brute.rs）
    Brute,
    /// 行単位の深さ優先探索（dfs.rs）
    Dfs,
    /// 予算付きの行単位 DP。帯の下界で枝刈りする（dp.rs）
    Dp,
}

/// OEIS A344719: 塗っていない全マスがちょうど 2 個の塗ったマスに隣接するよう塗る最小個数
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// 盤面サイズ n。--to と併用すると開始サイズ
    n: usize,
    /// 終了サイズ。n から to まで順に解く
    #[arg(long)]
    to: Option<usize>,
    /// 探索アルゴリズム
    #[arg(long, value_enum, default_value_t = Algo::Dp)]
    algo: Algo,
}

fn run(n: usize, algo: Algo) {
    let start = Instant::now();
    let (board, nodes) = match algo {
        Algo::Brute => (brute::solve(n), None),
        Algo::Dfs => {
            let (board, nodes) = dfs::solve_with_nodes(n);
            (board, Some(nodes))
        }
        Algo::Dp => {
            let board = dp::solve_by_budget(n, |budget, o| {
                let result = if o.board.is_some() {
                    "解あり"
                } else {
                    "解なし"
                };
                println!(
                    "budget {budget}: {result} (transitions: {}, max states: {})",
                    o.transitions, o.max_states
                );
            });
            (board, None)
        }
    };
    let elapsed = start.elapsed();
    assert!(board.is_valid(), "解が条件を満たしていない");
    print!("{board}");
    let count = board.count();
    match known(n) {
        Some(k) => println!("n = {n}: {count} (OEIS: {k})"),
        None => println!("n = {n}: {count}"),
    }
    if let Some(nodes) = nodes {
        println!("nodes: {nodes}");
    }
    println!("time: {elapsed:.2?}");
}

fn main() {
    let args = Args::parse();
    let to = args.to.unwrap_or(args.n);
    for n in args.n..=to {
        if n > args.n {
            println!();
        }
        run(n, args.algo);
    }
}
