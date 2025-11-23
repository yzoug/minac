# minac - `minac` Is Not A Chessboard

## What `minac` is

A text-based middleware to Lichess's board API, written in Rust.

Currently, the `minac` binary waits for input from `stdin`:

* moves are inputted as SAN (Standard Algebraic Notation). E.g. `d4`, `Nf4`, `Qxf7`...
* chess game status is recorded using the `chess` crate
* lichess API is used via the `lichess_api` crate for online games and for saving copies of offline games into a study
* you can play against another human or against Stockfish via the commandline

## Installation and usage

To compile it locally, you need [my fork of the `chess` crate](https://github.com/yzoug/chess).

Because I need a custom `chess` crate version, you also need to clone the `vampirc-uci` crate even though I did not modify it. Clone it [from here](https://github.com/vampirc/vampirc-uci) and modify its `Cargo.toml` as follows:

```diff
-chess = { version = "3.2", optional = true }
+chess = { path = "../chess", optional = true }
```

Finally, a Lichess token with the `challenge:read/write`, `board:play` and `study:write` permissions is needed: get it [from here](https://lichess.org/account/oauth/token) and define it in the $MINAC_LICHESS_TOKEN environment variable before running `cargo run`.

This software is still in alpha, and I don't much have experience with Rust. You've been warned ;).

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

All the buttons, except the clock buttons, could be replaced by a touchscreen. Adding voice control could also be possible. Everything should be handled by the `minac` binary.

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

* Stockfish integration: initial version is merged, we know need position evaluation on games when they are over, move by move, to identify errors/blunders and output them. [This page](https://github.com/official-stockfish/Stockfish/wiki/UCI-%26-Commands) has all the info I need.
* Lichess studies API: part of it is implemented in [this PR for my fork](https://github.com/yzoug/lichess-api/pull/2) of the [lichess\_api crate](https://github.com/ion232/lichess-api), code needs cleaning before submitting a PR upstream.

On the hardware side:

* Compiling for Raspberry Pi 3B+, same steps as described above: everything works, tested on Raspberry Pi OS Lite 64bit (i.e. ARMv8/AArch64). To compile, Rust (ofc) and the `openssl-dev` package need to be installed. Stockfish needs to be built from source for the ARMv8 architecture: download the sources for Stockfish 16 [from here](https://github.com/official-stockfish/Stockfish/archive/sf_16.zip) and run `make build ARCH=armv8`.
* Interfacing with buttons. What I'm thinking is a given button press goes directly to a first GPIO, my code starts counting time when the voltage is high on that pin. Each button is also relied to a second GPIO, but through condensators, different values per button. Then depending on how much milliseconds passed between the high detected on the first GPIO and the second one, I know which button is pressed. This makes it possible to have a bunch of buttons using only two pins. Maybe there are better ways? Anyway this is the only solution I could think of.
* For the sake of simplicity and since Stockfish needs some computing power, this project will probably not run on a weaker controller than Raspberry Pis. I'll probably use directly the [`rppal`](https://github.com/golemparts/rppal) crate, instead of going the `embedded-hal` route.

Starting to thinker about:

* 3D case design with Blender: something like [this](https://www.thingiverse.com/thing:3154742) to be 3D printed.

## About me

Lichess: @zoug

Mastodon: @zoug@infosec.exchange

Github: yzoug

https://zoug.fr

