# grid_2fair

English | [日本語](README.ja.md)

A Rust program that computes terms of [OEIS A344719](https://oeis.org/A344719).

Paint some cells of an n×n grid so that every unpainted cell is orthogonally adjacent to exactly two painted cells. a(n) is the minimum number of painted cells.

## Results

As of 2026-10-04 the OEIS entry lists a(1)..a(15) (a(12)..a(15) were computed by Bert Dobbelaere). This program reproduces a(1)..a(15) and extends the sequence.

| n | a(n) | Source |
|--:|--:|---|
| 1..15 | 1, 2, 5, 8, 11, 17, 21, 28, 35, 42, 51, 60, 69, 80, 91 | OEIS (reproduced) |
| 16 | 102 | this program |
| 17 | 115 | this program |
| 18 | 128 | this program |
| 19 | 141 | this program |
| 20 | ≥ 156 | this program (no solution up to budget 155, run not finished) |

Each a(n) is established by two checks:

- For every budget from the lower bound up to a(n)−1 the DP reports "infeasible", so a(n)−1 cells are not enough.
- A solution with a(n) cells is reconstructed and every cell is verified with `Board::is_valid`.

From n = 8 on, the first differences are 7, 7, 7, 9, 9, 9, 11, 11, 11, 13, 13, 13: each value repeats three times.

## Usage

Always use a release build. A debug build is orders of magnitude slower.

```sh
cargo run --release -- 16            # solve n = 16
cargo run --release -- 3 --to 18     # solve n = 3 through 18 in order
cargo run --release -- 16 --algo dp  # choose the algorithm (dfs | dp | brute, default dfs)
```

The output shows the verdict for each budget, the solution board (`#` = painted), the number of painted cells, and the elapsed time. The DFS also prints the number of search nodes; the DP prints the progress of the reconstruction.
The upper limit of n is 30 for dfs and dp, and 7 for brute. brute is practical only up to n = 5.

## Algorithm

Open [docs/algorithm.html](docs/algorithm.html) in a browser for an illustrated explanation with measurements and interactive demos. In short:

- Each row is a bit mask and rows are decided from the top down. Once two consecutive rows are fixed, most of the next row is forced (`forced_below`).
- For a budget B the search decides whether B cells suffice; the budget is raised one at a time from a lower bound, and the first feasible budget is a(n).
- The lower bound for the remaining rows is built from the minimum number of painted cells in horizontal strips of height up to 9 (computed by a column-wise DP).
- Before entering a row, the forced cells of the next row are computed. Dead ends and states whose count plus lower bound exceeds the budget are pruned (lookahead).
- Left-right mirror images are searched only once.
- The default DFS keeps only the current path in memory. The DP keeps one row of states in a hash table, keeps only two tables, and reconstructs the board by alternating-direction DP passes.

## Measurements

Single-threaded on a Core i7-12700F with 32 GB of RAM.

DFS (default):

| n | Total time | Nodes |
|--:|--:|--:|
| 13 | 0.16 s | 18,431 |
| 14 | 0.21 s | 197,804 |
| 15 | 0.29 s | 466,183 |

Most of this time is the roughly 0.1 s spent computing the lower bound.

DP:

| n | Total time | Max states | Peak memory |
|--:|--:|--:|--:|
| 16 | 0.48 s | 184,106 | |
| 17 | 6.3 s | 3,488,013 | |
| 18 | 7.1 s | 2,026,563 | |
| 19 | 21.5 s | 6,754,091 | 345 MB |

The DP's memory grows 5 to 8 times with each increase of n, while the DFS needs memory only for the current path.

## Layout

| File | Role |
|---|---|
| `src/dfs.rs` | Budgeted row-by-row DFS (default): check before entering a row (`admit`), recursion (`rec`) |
| `src/dp.rs` | Budgeted row-by-row DP: two-table DP (`forward`), feasibility test (`search`), reconstruction (`reconstruct`) |
| `src/bound.rs` | Strip lower bounds (`strip_min`, `lower_bounds`), shared by DFS and DP |
| `src/row.rs` | Shared row operations: forced cells (`forced_below`), state keys, mirroring |
| `src/brute.rs` | Brute force, used only for cross-checking |
| `src/board.rs` | Board representation and validity check |
| `src/lib.rs` | Known OEIS values |
| `src/main.rs` | Command-line interface |
| `examples/profile.rs` | Prints a per-row breakdown of the DP states |

## Tests

```sh
cargo test --release
```

The tests check that DFS and DP agree with each other and with the OEIS values up to n = 15, that brute force agrees for small n, that the lower bound never exceeds the answer, that a budget one below the answer is infeasible, and that merging mirrored states does not change the verdict.

## Solutions for the new terms

<details><summary>n = 16 (102 cells)</summary>

```
#..#.##.##.##.#.
.##..#..#..#..##
.#..#..#..#..#..
#..#..#..#..#..#
..#..#..#..#..##
##..#..#..#..#..
#..#..#..#..#..#
..#..#..#..#..##
##..#..#..#..#..
#..#..#..#..#..#
..#..#..#..#..##
##..#..#..#..#..
#..#..#..#..#..#
..#..#..#..#..#.
##..#..#..#..##.
.#.##.##.##.#..#
```

</details>

<details><summary>n = 17 (115 cells)</summary>

```
.#.##.##.##.##.#.
#..#..#..#..#..##
..#..#..#..#..#..
##..#..#..#..#..#
#..#..#..#..#..##
..#..#..#..#..#..
##..#..#..#..#..#
#..#..#..#..#..##
..#..#..#..#..#..
##..#..#..#..#..#
#..#..#..#..#..##
..#..#..#..#..#..
##..#..#..#..#..#
#..#..#..#..#..##
..#..#..#..#..#..
##..#..#..#..#..#
.#.##.##.##.##.#.
```

</details>

<details><summary>n = 18 (128 cells)</summary>

```
#.#.#.##.##.#..##.
..#.#..#..#..##..#
###..#..#..#..#..#
...#..#..#..#..##.
##..#..#..#..#..#.
..#..#..#..#..#..#
#..#..#..#..#..#..
##..#..#..#..#..##
..#..#..#..#..#..#
#..#..#..#..#..#..
##..#..#..#..#..##
..#..#..#..#..#..#
#..#..#..#..#..#..
.#..#..#..#..#..##
.##..#..#..#..#...
#..#..#..#..#..###
#..##..#..#..#.#..
.##..#.##.##.#.#.#
```

</details>

<details><summary>n = 19 (141 cells)</summary>

```
#.#.#.##.##.##.#..#
..#.#..#..#..#..##.
###..#..#..#..#..#.
...#..#..#..#..#..#
##..#..#..#..#..#..
..#..#..#..#..#..##
#..#..#..#..#..#..#
##..#..#..#..#..#..
..#..#..#..#..#..##
#..#..#..#..#..#..#
##..#..#..#..#..#..
..#..#..#..#..#..##
#..#..#..#..#..#..#
##..#..#..#..#..#..
..#..#..#..#..#..##
#..#..#..#..#..#...
.#..#..#..#..#..###
.##..#..#..#..#.#..
#..#.##.##.##.#.#.#
```

</details>
