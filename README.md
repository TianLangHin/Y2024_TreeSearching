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