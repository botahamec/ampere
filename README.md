# Ampere

This is my experiment with a Checkers AI, designed to be very fast. I want to eventually make a chess variant with playing cards, but to do that I would need to make an AI player. That sounds hard, so I decided to start with something simpler. And what can be simpler than Checkers?

Currently, the AI isn't very good. It's much better than me, but even simple Checkers AIs are able to beat it. I attribute this to the fact that my evaluation function is very simple. It only counts the number of pieces each player has. It doesn't attempt to account for moves which may be possible in the future. I think this will be the first thing I try to fix when I get back to working on this.

The search function is very fast. Here are the features implemented:
- transposition table
- alpha-beta pruning
- iterative deepening
- aspiration windows
- move ordering (using a lazily-evaluated selection sort)
- a 16-bit evaluation type

The move generation is also very fast thanks to:
- efficient bitboards (moving forward right means adding 1. moving left means adding 7)
- bitwise rotations
- an 8-bit Move type
- bitmasks for generating available moves
- storing the jump availability in the unused bits

On my machine (Ryzen 5 3600), the AI is able to look 20-ply into the future in less than one second. For reference, Stockfish can only look 18-ply into the future in this time.

Here are some features I'd like to implement in the future:
- better move ordering (low-depth search, triangular pv-table)
- extensions (capture, one-reply, passed piece, recapture, principal variation)
- reducations (fail-high, late-move, null-move
- principal variation search
- some best-first attempts
- multi-threaded search
- better evaluation
  - piece-square table
  - piece-structure heuristics
  - mobility
  - square control
  - razoring
  - RankCut
