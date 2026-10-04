use std::time::Instant;

use clap::{CommandFactory, Parser, ValueEnum, error::ErrorKind};
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

impl Algo {
    /// 扱える n の上限。brute は盤面全体を u64 のマスクで表し、
    /// dfs は 1 行を u64 で表し、dp は 2 行を u64 のキーに詰める
    fn max_n(self) -> usize {
        match self {
            Algo::Brute => 7,
            Algo::Dfs => 63,
            Algo::Dp => 30,
        }
    }

    /// コマンドラインでの名前（dp など）
    fn name(self) -> String {
        self.to_possible_value()
            .expect("全ての variant をコマンドラインで指定できる")
            .get_name()
            .to_owned()
    }
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
            let board = dp::solve_by_budget(
                n,
                |budget, o| {
                    let result = if o.min.is_some() {
                        "解あり"
                    } else {
                        "解なし"
                    };
                    println!(
                        "budget {budget}: {result} (transitions: {}, max states: {}, {:.2?})",
                        o.transitions,
                        o.max_states,
                        start.elapsed()
                    );
                },
                |fixed, transitions, max_states| {
                    println!(
                        "  復元 {fixed}/{n} 行 (transitions: {transitions}, max states: {max_states}, {:.2?})",
                        start.elapsed()
                    );
                },
            );
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
    let max = args.algo.max_n();
    let mut cmd = Args::command();
    if args.n == 0 || to > max {
        cmd.error(
            ErrorKind::ValueValidation,
            format!(
                "--algo {} では n と --to を 1..={max} の範囲で指定してください",
                args.algo.name()
            ),
        )
        .exit();
    }
    if to < args.n {
        cmd.error(
            ErrorKind::ValueValidation,
            format!("--to ({to}) は n ({}) 以上にしてください", args.n),
        )
        .exit();
    }
    for n in args.n..=to {
        if n > args.n {
            println!();
        }
        run(n, args.algo);
    }
}
