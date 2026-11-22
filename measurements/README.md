# Measurements

Produced by `node-circuit/examples/scaling.rs` with the real prover (Halo2,
IPA over the Pasta curves) on the machine described in `machine.txt`:

```bash
cargo run --release -p node-circuit --example scaling -- all --reps 2
```

`threads`, `k-curve`, `size` or `grid` instead of `all` runs one experiment. All times
are wall-clock seconds.

## `threads.csv`: one proof on 1 … N threads

The energy tree (50 nodes) in one proof (`k = 16`), proved in a thread pool
of `threads` threads.

| Column | Meaning |
|---|---|
| `tree`, `nodes`, `k` | the tree, its node count, and `log2` of the circuit's rows |
| `threads` | size of the prover's thread pool |
| `repetition` | 0, 1, … |
| `prove_s` | `create_proof` time (key generation excluded) |

## `sharding.csv`: shards

Synthetic trees of the policy

```prolog
t(0, X) :- base(X).
t(N, node(L, R)) :- N > 0, M is N - 1, t(M, L), t(M, R).
```

where the proof of `t(d, T)` has `5·2^d − 3` nodes (`synthetic-d<d>`).

- **`k_curve`:** the tree of depth 6 (317 nodes), split into shards that just
  fit `2^k` rows, for `k = 19 … 14`. At `k = 19` the whole tree is one proof.
- **`size`:** depths 3 … 7 (37 … 637 nodes), in shards that fit `2^15` rows,
  and in one proof while that fits `2^19` rows.

The planner keeps subtrees together (`proof_witness::plan`). The circuit has
`capacity` node places and as many link places as the plan's busiest shard
needs.

| Column | Meaning |
|---|---|
| `experiment` | `k_curve` or `size` |
| `tree`, `nodes` | the tree and its node count |
| `k` | `log2` of the rows of each shard circuit |
| `capacity`, `incoming`, `outgoing` | node places and link places per shard |
| `shards`, `links` | `K`, and the links between the shards |
| `repetition` | 0, 1, … |
| `keygen_s` | proving key generation, for the root shard and (if `K > 1`) the others; once per policy and sizes |
| `prove_total_s` | all shards one after the other, each with every thread: the total work |
| `prove_max_s` | the slowest shard of that run: the wall-clock time on `K` machines like this one |
| `parallel_threads` | threads per shard when all shards run at once on this machine |
| `parallel_wall_s` | all shards at once on this machine, in pools of `parallel_threads` threads (waves if `K` exceeds the thread count): an emulation of `K` smaller machines that share memory bandwidth |
| `proof_bytes` | the size of all shard proofs together |
| `verify_s` | verifying all shard proofs and matching their links; the verifier's key generation excluded |

## `grid.csv`: threads per shard × shards at a time

The energy tree (50 nodes) split into shards that fit `2^k` rows for
`k = 16, 15, 14` (`K = 1, 2, 7`; `k = 13` does not fit a node with the links
the root's shard needs). Each plan is proved with `c` shards at a time, each
in a pool of `t` threads, for `t = 1, 2, 4, …` and `c = 1, 2, 4, …, K` with
`c · t` at most the thread count. With `c < K` the shards queue: a worker that
is done takes the next one, about `⌈K / c⌉` rounds. Keys are generated once
per plan, before the timing.

| Column | Meaning |
|---|---|
| `tree`, `nodes`, `k` | the tree, its node count, `log2` of the rows of each shard |
| `capacity`, `incoming`, `outgoing` | node and link places per shard |
| `shards`, `links` | `K`, and the links between the shards |
| `threads_per_shard` | `t` |
| `concurrent_shards` | `c` |
| `threads_used` | `c · t` |
| `repetition` | 0, 1, … |
| `wall_s` | wall-clock time until every shard is proved |

## Reading the numbers

- `prove_max_s` assumes one machine per shard, each like this one. The
  emulation `parallel_wall_s` runs on one machine, so it shows how much of
  that survives on fixed hardware.
- `k` grows in steps: a shard whose rows just exceed `2^k` costs about twice
  as much. The plans here pick the largest shards that fit each `k`.
- Proof size and verification time grow with `K` (one IPA proof each); this
  is the price of horizontal scaling without recursive aggregation.
