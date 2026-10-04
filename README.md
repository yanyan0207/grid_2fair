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
| 20 | 156 | this program |
| 21 | 171 | this program |
| 22 | 186 | this program |

Each a(n) is established by two checks:

- A solution with a(n) cells is found and every cell is verified with `Board::is_valid`.
- An exhaustive search with budget a(n)−1 finds no solution, so a(n)−1 cells are not enough (and therefore no smaller number is either).

a(16)..a(22) were obtained independently by at least two of the algorithms below (DP, DFS, stripes-dfs).

See [Conjectures](#conjectures) for the patterns these values suggest.

## Conjectures

The computed values and solutions suggest two conjectures. Neither is proven.

**Conjecture 1 (formula).** For n ≥ 7,

```
a(n) = ⌈(n+2)² / 3⌉ − 6
```

- It matches all 16 values for n = 7..22. Equivalently, from n = 8 on the first differences are 7, 7, 7, 9, 9, 9, 11, 11, 11, 13, 13, 13, 15, 15, 15: each value repeats three times.
- The leading term n²/3 has a simple reason. Each unpainted cell touches exactly 2 painted cells and each painted cell touches at most 4 cells, so at least a third of the cells must be painted. The (n+2)² form and the −6 presumably come from the edges.

**Conjecture 2 (shape of solutions).** For every n ≥ 6 there is an optimal solution in which every row, apart from 2 rows at the top and bottom and 2 columns at the left and right, is a diagonal stripe of period 3 (only columns c ≡ s mod 3 painted, with s depending on the row).

- For n = 6..22, the optimal solution found by the DFS has this shape. This shows such a solution exists for those n; it does not show that every optimal solution is striped.
- The default algorithm, stripes-dfs, is designed around this conjecture (see [Algorithm](#algorithm)).

## Usage

Always use a release build. A debug build is orders of magnitude slower.

```sh
cargo run --release -- 16                # solve n = 16
cargo run --release -- 3 --to 22         # solve n = 3 through 22 in order
cargo run --release -- 16 --algo dfs     # choose the algorithm (stripes-dfs | dfs | dp | brute, default stripes-dfs)
```

The output shows the progress of the search, the solution board (`#` = painted), the number of painted cells, and the elapsed time. After each n, the results so far are printed in b-file form (`n a(n)`) and as a comma-separated DATA line, so a run can be stopped with Ctrl-C at any point.
The upper limit of n is 30 for stripes-dfs, dfs and dp, and 7 for brute. brute is practical only up to n = 5.

## Algorithm

Open [docs/algorithm.html](docs/algorithm.html) in a browser for an illustrated explanation with measurements and interactive demos. In short:

- Each row is a bit mask and rows are decided from the top down. Once two consecutive rows are fixed, most of the next row is forced (`forced_below`).
- For a budget B the search decides whether B cells suffice. The lower bound for the remaining rows is built from the minimum number of painted cells in horizontal strips of height up to 9 (computed by a column-wise DP).
- Before entering a row, the forced cells of the next row are computed. Dead ends and states whose count plus lower bound exceeds the budget are pruned (lookahead). Left-right mirror images are searched only once.
- **stripes-dfs (default)**: optimal solutions are diagonal stripes of period 3 except near the edges. A solution is first built among boards whose rows are stripes except for 2 rows and 2 columns at each edge: two middle rows are chosen and the board is extended row by row toward both edges, with memoization. Its count U is then proven minimal by an exhaustive DFS with budget U−1. If that DFS finds a solution, its count becomes the new U.
  This algorithm is designed around [Conjecture 2](#conjectures). The answer does not depend on it, because minimality is always proven by the unrestricted DFS. Only the speed does: if no striped solution reaches a(n), the DFS has to search a feasible budget and takes about as long as dfs.
- **dfs**: raises the budget one at a time from the lower bound; the first feasible budget is a(n). Keeps only the current path in memory.
- **dp**: the same search done breadth-first with a table of states (current row, forced cells below it). It keeps two tables and reconstructs the board afterwards. Used for cross-checking and profiling.

## Measurements

Single-threaded on a Core i7-12700F with 32 GB of RAM.

| n | stripes-dfs (default) | dfs | dp |
|--:|--:|--:|--:|
| 19 | 2.2 s | 5.3 s | 13.2 s |
| 20 | 12.2 s | 54 s | |
| 21 | 46.4 s | 168 s | |
| 22 | 164 s | 552 s | |

With stripes-dfs, building the striped solution takes under 5 s even for n = 22; the rest is the exhaustive search with budget a(n)−1. The dfs and stripes-dfs searches keep only the current path in memory.

## Layout

| File | Role |
|---|---|
| `src/stripes_dfs.rs` | stripes-dfs (default): builds a striped solution (`construct`) and proves it minimal with the DFS |
| `src/stripes.rs` | Striped rows: test (`is_striped`) and enumeration (`rows`) |
| `src/dfs.rs` | Budgeted row-by-row DFS: check before entering a row (`admit`), recursion (`rec`) |
| `src/dp.rs` | Budgeted row-by-row DP: table of (row, forced) states (`forward`), feasibility test (`search`), reconstruction (`reconstruct`) |
| `src/bound.rs` | Strip lower bounds (`strip_min`, `lower_bounds`), shared by DFS and DP |
| `src/row.rs` | Shared row operations: forced cells (`forced_below`), state keys, mirroring |
| `src/brute.rs` | Brute force, used only for cross-checking |
| `src/board.rs` | Board representation and validity check |
| `src/lib.rs` | Known OEIS values |
| `src/main.rs` | Command-line interface |

## Tests

```sh
cargo test --release
```

The tests check that stripes-dfs, DFS and DP agree with each other and with the OEIS values up to n = 15, that brute force agrees for small n, that the stripe margin does not change the answer, that the lower bound never exceeds the answer, that a budget one below the answer is infeasible, and that merging mirrored states does not change the verdict.

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

<details><summary>n = 20 (156 cells)</summary>

```
#.#.#.##.##.##.##.#.
..#.#..#..#..#..#..#
###..#..#..#..#..#..
...#..#..#..#..#..##
##..#..#..#..#..#..#
..#..#..#..#..#..#..
#..#..#..#..#..#..##
##..#..#..#..#..#..#
..#..#..#..#..#..#..
#..#..#..#..#..#..##
##..#..#..#..#..#..#
..#..#..#..#..#..#..
#..#..#..#..#..#..##
##..#..#..#..#..#..#
..#..#..#..#..#..#..
#..#..#..#..#..#..##
##..#..#..#..#..#..#
..#..#..#..#..#..#..
#..#..#..#..#..#..##
.#.##.##.##.##.##.#.
```

</details>

<details><summary>n = 21 (171 cells)</summary>

```
.##..#.##.##.##.##.#.
#..##..#..#..#..#..##
#..#..#..#..#..#..#..
.##..#..#..#..#..#..#
.#..#..#..#..#..#..##
#..#..#..#..#..#..#..
..#..#..#..#..#..#..#
##..#..#..#..#..#..##
#..#..#..#..#..#..#..
..#..#..#..#..#..#..#
##..#..#..#..#..#..##
#..#..#..#..#..#..#..
..#..#..#..#..#..#..#
##..#..#..#..#..#..##
#..#..#..#..#..#..#..
..#..#..#..#..#..#..#
##..#..#..#..#..#..#.
#..#..#..#..#..#..##.
..#..#..#..#..#..#..#
##..#..#..#..#..##..#
.#.##.##.##.##.#..##.
```

</details>

<details><summary>n = 22 (186 cells)</summary>

```
#..#.##.##.##.##.##.#.
.##..#..#..#..#..#..##
.#..#..#..#..#..#..#..
#..#..#..#..#..#..#..#
..#..#..#..#..#..#..##
##..#..#..#..#..#..#..
#..#..#..#..#..#..#..#
..#..#..#..#..#..#..##
##..#..#..#..#..#..#..
#..#..#..#..#..#..#..#
..#..#..#..#..#..#..##
##..#..#..#..#..#..#..
#..#..#..#..#..#..#..#
..#..#..#..#..#..#..##
##..#..#..#..#..#..#..
#..#..#..#..#..#..#..#
..#..#..#..#..#..#..##
##..#..#..#..#..#..#..
#..#..#..#..#..#..#..#
..#..#..#..#..#..#..#.
##..#..#..#..#..#..##.
.#.##.##.##.##.##.#..#
```

</details>
