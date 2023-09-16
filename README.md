# minac - `minac` Is Not A Chessboard

## What `minac` is

A text-based middleware to Lichess's board API, written in Rust.

Currently, the `minac` binary waits for input from `stdin`:

* white makes first move as SAN (Standard Algebraic Notation)
* chess game status is recorded using the `chess` crate
* for wrong move, refuses the input, asks to do another one
* when checkmate is on the board, displays the full game as PGN.

To compile it locally, you need my fork of the `chess` crate:

* This [PR adds support for PGN](https://github.com/jordanbray/chess/pull/71).
* Checkout the branch and put the repo as `../chess-yzoug-fork`: see `Cargo.toml`.

To use the online mode, a Lichess token with the `challenge:read/write` and `board:play` permissions is needed: write this token in the `token.secret` file at the root of the project, before running `cargo run`.

This software is still in alpha, and I don't have experience with Rust. You've been warned ;).

## What `minac` wants to be

A 3D printed chess clock that makes it easy to play on a physical board connected to Lichess, and can serve as a game notation device that create studies, analyzes games etc.

The final object will probably be:

* A Raspberry Pi Zero
* A speaker
* A 3D printed casing
* Small LCD screen
* Buttons connected to the a GPIO pin (voltage dividers)
    * two clock buttons
    * ranks: from 1 to 8 included, optionally 0 and 9 also for time settings
    * files: A to H included
    * pieces: king, queen, rook, bishop, knight
    * online mode button (lichess logo), offline mode button (stockfish logo)

All the buttons, except the clock buttons, could be replaced by a touchscreen. Everything should be handled by the `minac` binary.

Wanted features:

* Audio clips (confirm inputted moves, error when wrong move is pressed...).
* Online mode
    * Lichess Board API to play using an online account
    * Choose ranked or friendly and time settings
    * Play against online bots
* Offline mode against bot
    * Play against Stockfish and setting its level, possibly other bots
    * Choose side, time settings
* Offline 2 players mode
    * Standard chess clock features
    * Save game as PGN
    * Open studies on Lichess

This is v5 I'd say, we're not exactly there.

## Licence

This project is licensed with the Affero GPLv3. See LICENSE.md for the full license, or [this page](https://choosealicense.com/licenses/agpl-3.0/) for a quick recap.

## Contributing

Feel free to send your enhancements and patches as PRs, or open issues.

## Roadmap

Currently working on:

* Handling of game aborts, draws etc.
* Small analysis (what the lichess API sends) at end of game

Starting to thinker about:

* 3D case design
* Compiling for Raspberry Pi Zero

## About me

Lichess: @zoug

Mastodon: @zoug@infosec.exchange

Github: yzoug

https://zoug.top

