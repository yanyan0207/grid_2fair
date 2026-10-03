use clap::Parser;
use grid_2fair::{brute, known};

/// OEIS A344719: 塗っていない全マスがちょうど 2 個の塗ったマスに隣接するよう塗る最小個数
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// 盤面サイズ n
    n: usize,
}

fn main() {
    let args = Args::parse();
    let board = brute::solve(args.n);
    print!("{board}");
    let count = board.count();
    match known(args.n) {
        Some(k) => println!("n = {}: {count} (OEIS: {k})", args.n),
        None => println!("n = {}: {count}", args.n),
    }
}
