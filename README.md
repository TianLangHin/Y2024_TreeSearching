# Y2024_TreeSearching

An environment to analyse and test different game tree searching algorithms under different game tree topologies.

## Algorithms

Currently, the algorithms tested are as delineated in [Muszycka and Shinghal (1985)](https://ieeexplore.ieee.org/document/6313374), implemented using Rust generics.

All game trees that are tested are defined in `src/games/`, and must satisfy the traits
defined in `src/prelude.rs`.

The algorithms as described in Muszycka and Shinghal (1985) are implemented under the following identifiers:

* Algorithm A is `branch_and_bound`.
* Algorithm B is `alpha_beta`.
* Algorithm C is `p_alpha_beta`.
* Algorithm D is `pvs`.
* Algorithm E is `scout`.
* Algorithm F is `sss`, but is implemented from [Stockman's (1979)](https://www.sciencedirect.com/science/article/abs/pii/000437027990016X) original formulation, as no new formulation was given in Muszycka and Shinghal (1985).

## Game Tree Topologies

Currently, the following game trees are implemented in `src/games`.

* A representation of the sample game tree in Stockman's (1979) original proposal of SSS*, defined in `stockman.rs`.
* A representation of Ultimate Tic-Tac-Toe using bitboards, defined in `ut3.rs`.
* A representation of Chess using bitboards (particularly magic lookups), defined in `chess.rs`.
* A representation of a uniform game tree of constant branching factor 2, and has randomly assigned node values based on the `rand_chacha` crate, defined in `uniform_2b_wide.rs`.
* A representation of a uniform game tree with node values assigned by the `unordered-independent` scheme as used in Muszycka and Shinghal (1985), using the `rand_chacha` crate in conjunction with the Fisher-Yates Shuffle, defined in `hypothetical_tree.rs`.

## Main Program

The driver code in `src/main.rs` tests each of the game tree topologies with each of the six algorithms,
and the final 24 entries are averaged results of 1,000,000 test cases for each of the 24 settings
that Muszycka and Shinghal (1985) test.
For each setting, the average number of leaf nodes evaluated and average time used is outputted,
with resolutions at milliseconds, microseconds and nanoseconds.