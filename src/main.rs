use std::time::Instant;

use clap::{CommandFactory, Parser, ValueEnum, error::ErrorKind};
use grid_2fair::{brute, dfs, dp, known};

/// Search algorithm
#[derive(Clone, Copy, Debug, ValueEnum)]
enum Algo {
    /// Brute force over all boards (brute.rs)
    Brute,
    /// Budgeted row-by-row depth-first search pruned by strip lower bounds (dfs.rs)
    Dfs,
    /// Budgeted row-by-row DP pruned by strip lower bounds (dp.rs)
    Dp,
}

impl Algo {
    /// 扱える n の上限。brute は盤面全体を u64 のマスクで表し、
    /// dfs は 1 行を u64 で表し、dp は 2 行を u64 のキーに詰める
    fn max_n(self) -> usize {
        match self {
            Algo::Brute => 7,
            Algo::Dfs => 30,
            Algo::Dp => 30,
        }
    }

    /// コマンドラインでの名前（dp など）
    fn name(self) -> String {
        self.to_possible_value()
            .expect("every variant is selectable on the command line")
            .get_name()
            .to_owned()
    }
}

/// OEIS A344719: minimum number of painted cells in an n x n grid so that
/// every unpainted cell is orthogonally adjacent to exactly two painted cells
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Board size n (the first size when used with --to)
    n: usize,
    /// Last board size; solves n, n+1, ..., to in order
    #[arg(long)]
    to: Option<usize>,
    /// Search algorithm
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
                        "feasible"
                    } else {
                        "infeasible"
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
                        "  reconstructed {fixed}/{n} rows (transitions: {transitions}, max states: {max_states}, {:.2?})",
                        start.elapsed()
                    );
                },
            );
            (board, None)
        }
    };
    let elapsed = start.elapsed();
    assert!(
        board.is_valid(),
        "the solution does not satisfy the condition"
    );
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
                "with --algo {}, n and --to must be in 1..={max}",
                args.algo.name()
            ),
        )
        .exit();
    }
    if to < args.n {
        cmd.error(
            ErrorKind::ValueValidation,
            format!("--to ({to}) must be at least n ({})", args.n),
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
