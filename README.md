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
cargo run --release -- 8 --algo dfs  # choose the algorithm (dp | dfs | brute, default dp)
```

The output shows the verdict for each budget, the progress of the reconstruction, the solution board (`#` = painted), the number of painted cells, and the elapsed time.
The upper limit of n is 30 for dp, 63 for dfs and 7 for brute. In practice dp handles n ≤ 19, dfs n ≤ 9 and brute n ≤ 5.

## Algorithm

Open [docs/algorithm.html](docs/algorithm.html) in a browser for an illustrated explanation with measurements. In short:

- Each row is a bit mask and rows are decided from the top down. Once two consecutive rows are fixed, most of the next row is forced (`forced_below`).
- A row-by-row DP whose state is the last two rows. For a budget B it decides whether B cells suffice; the budget is raised one at a time from a lower bound.
- The lower bound for the remaining rows is built from the minimum number of painted cells in horizontal strips of height up to 9 (computed by a column-wise DP).
- States that coincide under a left-right reflection are merged.
- Only two row tables are kept; the board is reconstructed by alternating-direction DP passes.

## Measurements

Single-threaded on a Core i7-12700F with 32 GB of RAM.

| n | Total time | Max states | Peak memory |
|--:|--:|--:|--:|
| 16 | 0.75 s | 3,324,051 | |
| 17 | 15.2 s | 54,660,545 | |
| 18 | 19.4 s | 63,314,198 | 4.4 GB |
| 19 | 5,792 s | 268,439,434 | |

At n = 19 the state table no longer fits in memory and the time per transition is more than 100 times that of n = 18. Going beyond n = 20 requires a smaller state table.

## Layout

| File | Role |
|---|---|
| `src/dp.rs` | Budgeted row DP: strip lower bounds (`strip_min`, `lower_bounds`), two-table DP (`forward`), feasibility test (`search`), reconstruction (`reconstruct`) |
| `src/dfs.rs` | The original row-by-row DFS. `forced_below` is shared with the DP |
| `src/brute.rs` | Brute force, used only for cross-checking |
| `src/board.rs` | Board representation and validity check |
| `src/lib.rs` | Known OEIS values |
| `src/main.rs` | Command-line interface |

## Tests

```sh
cargo test
```

The tests check that brute force, DFS and DP agree with each other and with the OEIS values, that the lower bound never exceeds the answer, that a budget one below the answer is infeasible, and that merging mirrored states does not change the verdict.

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
