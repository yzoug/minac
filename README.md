# minac - `minac` Is Not A Chessboard

## What `minac` is

A text-based middleware to Lichess's board API, written in Rust.

Currently, the `minac` binary waits for input from `stdin`:

* moves are inputted as SAN (Standard Algebraic Notation). E.g. `d4`, `Nf4`, `Qxf7`...
* chess game status is recorded using the `chess` crate
* lichess API is used via the `lichess_api` crate for online games and for saving copies of offline games into a study

## Installation and usage

To compile it locally, you need [my fork of the `chess` crate](https://github.com/yzoug/chess) and [my fork of the lichess-api crate](https://github.com/yzoug/lichess-api).

Most changes are merged upstream for the Lichess API (hopefully all when I clean up [this PR](https://github.com/yzoug/lichess-api/pull/2) and submit upstream). For the `chess` crate, I need support for PGN that I added in [this PR](https://github.com/jordanbray/chess/pull/71), however it doesn't seem like this PR will be merged (has been open for literally years with no feedback).

You hence need to clone both my forks and put them somewhere `minac` can find them. By default, this is next to the `minac` folder you cloned, see `Cargo.toml`.

Because I need a custom `chess` crate version, you also need to clone the `vampirc-uci` crate even though I did not modify it. Clone it [from here](https://github.com/vampirc/vampirc-uci) and modify its `Cargo.toml` as follows:

```diff
-chess = { version = "3.2", optional = true }
+chess = { path = "../chess", optional = true }
```

Finally, a Lichess token with the `challenge:read/write`, `board:play` and `study:write` permissions is needed: get it [from here](https://lichess.org/account/oauth/token) and write it in the `token.secret` file at the root of the project, before running `cargo run`.

This software is still in alpha, and I don't have experience with Rust. You've been warned ;).

## What `minac` wants to be

A 3D printed chess clock that makes it easy to play on a physical board connected to Lichess, and can serve as a game notation device that create studies, analyzes games etc.

The final object will probably be:

* A Raspberry Pi
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

* Stockfish integration: initial version is merged, we know need position evaluation on games when they are over, move by move, to identify errors/blunders and output them.
* Lichess studies API: part of it is implemented in [this PR for my fork](https://github.com/yzoug/lichess-api/pull/2) of the [lichess\_api crate](https://github.com/ion232/lichess-api), code needs cleaning before submitting a PR upstream.

On the hardware side:

* Compiling for Raspberry Pi: the target is either arm-unknown-linux-gnueabihf or arm-unknown-linux-gnueabi
* the embedded hello world: blink a LED with Rust
* Display the voltage received by a GPIO pin

Starting to thinker about:

* 3D case design with Blender

## About me

Lichess: @zoug

Mastodon: @zoug@infosec.exchange

Github: yzoug

https://zoug.top

