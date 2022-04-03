minac - minac Is Not A Chessboard
================

High-level description
-----------------

The final object will probably be:

* A Raspberry Pi
* A Raspberry Pi speaker (Pi-hat, also contains a microphone)
* A 3D printed casing
* Buttons connected to the GPIO pins
    * ranks: from 1 to 8 included, optionally 0 and 9 also to be able to choose time settings
    * files: A to H included
    * pieces: king, queen, rook, bishop, knight
    * online mode (lichess logo), offline mode (stockfish logo)
    * microphone button?
* A small screen for a clock

The `minac` binary runs at boot and handles the Pi's GPIO buttons:

* Plays audio clips (greetings, explaining each menu, all moves)
* Online mode
    * If more than 1 account, choose its number
    * Choose ranked or friendly and time settings
* Offline mode
    * Choose Stockfish level, possibly other bots
    * Choose side, time settings
* Input the move when it's your turn
    * from GPIO buttons (select piece and destination square)
    * from microphone? a FOSS project at best, or using an online API (Google knowing my chess level is fine by me)
* Post-game report
    * Stockfish high-level game analysis as audio clips (critical errors, suggested lines etc)

This is v5.0.0 probably. Still very far.

minac
---------------

Version 0.1.0

Currently:
* Replace all vocal messages with log prints
* Wait for input from stdin:
    * white makes first move as SAN (Standard Algebraic Notation)
    * chess game status is recorded using the chess crate
    * for wrong move, refuses the input, asks to do another one
* when checkmate is on the board, displays the full game as PGN.
    * the chess crate has been forked for this. The [PR is here](https://github.com/jordanbray/chess/pull/71).
    * for now, clone [my forked repo](https://github.com/yzoug/chess) and switch on the "game-as-pgn" branch; it should be named "chess-yzoug-fork" and in the parent directory of this project (or modify the path specified in Cargo.toml)

Currently working on:
* Online game on Lichess. Objective: play a full game on Lichess using the board:play API through the current commandline interface
    * Personal token api needs perm challenge:read/write and board:play, use as `curl https://lichess.org/api/account -H "Authorization: Bearer {token}"`
    * board:play primarily: [doc](https://lichess.org/api#tag/Board)

In the future:
* Rust's GPIO crate provides what is needed to read values of different button presses : https://docs.rs/gpio/0.4.1/gpio/
* Create a dummy circuit board with 4 buttons, get the inputs

Licence
--------------

This project is licensed with the Affero GPLv3. See LICENSE.md for the full license, or [this page](https://choosealicense.com/licenses/agpl-3.0/) for a quick recap.

Contributing
--------------

Feel free to send your enhancements and patches as PRs, or open issues.

About me
-------------

Lichess: @zoug
Reddit, Twitter, Github: @yzoug
Blog: https://zoug.top

