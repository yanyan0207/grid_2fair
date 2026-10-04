use std::time::Instant;

use clap::{CommandFactory, Parser, ValueEnum, error::ErrorKind};
use grid_2fair::stripes_dfs::Phase;
use grid_2fair::{brute, dfs, dp, known, stripes, stripes_dfs};

/// Search algorithm
#[derive(Clone, Copy, Debug, ValueEnum)]
enum Algo {
    /// Brute force over all boards (brute.rs)
    Brute,
    /// Budgeted row-by-row depth-first search pruned by strip lower bounds (dfs.rs)
    Dfs,
    /// Budgeted row-by-row DP pruned by strip lower bounds (dp.rs)
    Dp,
    /// Find a solution among striped rows, then prove one fewer cell is infeasible (stripes_dfs.rs)
    StripesDfs,
}

impl Algo {
    /// 扱える n の上限。brute は盤面全体を u64 のマスクで表し、
    /// dfs は 1 行を u64 で表し、dp は 2 行を u64 のキーに詰める
    fn max_n(self) -> usize {
        match self {
            Algo::Brute => 7,
            Algo::Dfs => 30,
            Algo::Dp => 30,
            Algo::StripesDfs => 30,
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
    #[arg(long, value_enum, default_value_t = Algo::Dfs)]
    algo: Algo,
    /// Rows and columns left free at each edge when --algo stripes-dfs restricts rows to stripes
    #[arg(long, default_value_t = stripes::DEFAULT_MARGIN)]
    margin: usize,
}

/// n を解いて結果を表示し、a(n) を返す
fn run(n: usize, algo: Algo, margin: usize) -> usize {
    let start = Instant::now();
    let (board, nodes) = match algo {
        Algo::Brute => (brute::solve(n), None),
        Algo::Dfs => {
            let (board, nodes) = dfs::solve_by_budget(n, |budget, feasible, nodes| {
                let result = if feasible { "feasible" } else { "infeasible" };
                println!(
                    "budget {budget}: {result} (nodes: {nodes}, {:.2?})",
                    start.elapsed()
                );
            });
            (board, Some(nodes))
        }
        Algo::StripesDfs => {
            let (board, nodes) = stripes_dfs::solve_by_budget(
                n,
                margin,
                |phase, budget, feasible, nodes| match phase {
                    Phase::Construct => println!(
                        "construct: {budget} cells with striped rows (states: {nodes}, {:.2?})",
                        start.elapsed()
                    ),
                    Phase::Prove => {
                        let result = if feasible { "feasible" } else { "infeasible" };
                        println!(
                            "prove budget {budget}: {result} (nodes: {nodes}, {:.2?})",
                            start.elapsed()
                        );
                    }
                },
            );
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
    count
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
    let mut results = Vec::new();
    for n in args.n..=to {
        if n > args.n {
            println!();
        }
        results.push((n, run(n, args.algo, args.margin)));
        // 1 つ解くたびに、ここまでの結果をまとめて出す（b-file の形式と、OEIS の DATA 欄の形式）。
        // 途中で止めても、最後に出たまとめをそのまま貼れる
        print_results(&results);
    }
}

/// ここまでに解いた (n, a(n)) を、b-file の形式と OEIS の DATA 欄の形式で出す
fn print_results(results: &[(usize, usize)]) {
    println!("== results (n a(n)) ==");
    for (n, a) in results {
        println!("{n} {a}");
    }
    let data: Vec<String> = results.iter().map(|(_, a)| a.to_string()).collect();
    println!("== data ==");
    println!("{}", data.join(", "));
}
