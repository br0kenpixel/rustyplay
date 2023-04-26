<p align="center">
  <img width="460" height="300" src="https://github.com/br0kenpixel/rustyplay/blob/e01ac615b1e6b7cfb640e4426093c2a6787c1e89/img/tui.gif">
</p>
<p align="center">
  <strong>br0kenpixel's Music Player</strong>
</p>
<p align="center">
  <em>A terminal based music player with time-synced lyrics support.</em>
</p>

# Intro
This is a remade version of the original, which was written in C.  
⚠️ Playlist features (`Next`, `Previous`) are not implemented yet.

## Parts
- [`src/main.rs`](src/main.rs) - Contains the main entry point. You should start exploring from here.
- [`src/audioinfo.rs`](src/audioinfo.rs) - Provides implementations for reading metadata from audio files.
- [`src/lyrics.rs`](src/lyrics.rs) - The lyrics "engine."
- [`src/lyrics_parse.rs`](src/lyrics_parse.rs) - The lyrics parser.
- [`src/player.rs`](src/player.rs) - Provides implementations for controlling the audio player.
- [`src/display.rs`](src/display.rs) - Provides a high-level abstraction layer for creating and managing the UI.
- [`src/timer.rs`](src/timer.rs) - Provides a simple timer/countdown object.
- [`src/scrolledbuf.rs`](src/scrolledbuf.rs) - Provides an object for scrolling text.

## Building
First make sure you have the Rust toolchain installed.  
If not, use the following command on *nix based systems:  
```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

To run the player follow these steps:
1. Clone this repo
    - `git clone https://github.com/br0kenpixel/rustyplay`
2. Build the binary
    - `cargo build --release`
    - > ⚠️ It's highly recommended to build in release mode for better performance!
3. Run the binary like this:
    - `musicplayer [FILE]`
      - Example:
      - `musicplayer call_me.wav`
4. You can also build the documentation:
    - `cargo doc --open`

## Supported audio formats
- WAV
- OGG
- FLAC
  - > ~~⚠️ FLAC support is temporarily disabled due to issues with playback.~~
  - > ~~✅ FLAC playback has been fixed thanks to [this](https://docs.rs/rusty_audio/1.4.0/src/rusty_audio/lib.rs.html#85)!~~
  - > ✅ FLAC playback has been fixed by using optimizations instead of the fix mentioned above

## Supported systems:
As of now, it was only tested on macOS Monterey 12.6.1 (Intel). But theoretically it should work on any other OS, as all dependencies have cross-platform support.

# Dependencies
- [`rodio`](https://crates.io/crates/rodio)
  - An audio playback library
- [`sndfile`](https://crates.io/crates/sndfile)
  - Used to read metadata from audio files
- [`ncurses`](https://crates.io/crates/ncurses)
  - A popular terminal UI library
- [`pausable_clock`](https://crates.io/crates/pausable_clock)
  - Provides a pausable/resumable clock type
- [`serde`](https://crates.io/crates/serde)
  - A data serialization/deserialization framework
- [`serde_json`](https://crates.io/crates/serde_json)
  - Allows serialization/deserialization to/from JSON using `serde`.

# Lyrics
The time-synced lyrics are provided by Spotify/Musixmatch. In order to be able to use this feature, you must obtain a JSON file containing the time-synced lyrics data. Such data can be obtained by using either [`akashrchandran/spotify-lyrics-api`](https://github.com/akashrchandran/spotify-lyrics-api) or [`br0kenpixel/spotify-lyrics-api-rust`](https://github.com/br0kenpixel/spotify-lyrics-api-rust).
"End times" are also supported under certain conditions.

## Setting up
First, you need to use one of the tools listed above to obtain the lyrics data from Spotify. You'll need to save this data into a `.json` file. __This file must be located in the same directory as the audio file it "belongs" to!__  
For example, if you run `musicplayer Documents/Music/hello.wav` then `Documents/Music/hello.json` __must__ be a valid path and this file must contain the lyrics data obtained from Spotify. If this `.json` file does not exist, lyrics functionality will be disabled, however playback will work. If the `.json` file contains invalid data, the program will [`panic!()`](https://doc.rust-lang.org/std/macro.panic.html).

## "End time" support
So far I haven't noticed any lyrics data with `endTimeMs` set, however if the lyrics contain a line with a singe `♪` character (or is empty), the lyrics parser will automatically "adjust" the lyrics data. This line will be ignored and it's `startTimeMs` is changed to the previous line's `endTimeMs`.

# Documentation
You can use `cargo doc` to generate the documentation.  
The "homepage" of the documentation is `target/doc/musicplayer/index.html`.

# FAQ/Troubleshooting
<details>
  <summary>Why are MP3s and M4As not supported if rodio supports them?</summary>
  
  Even though `rodio` can play these files, the problem is `sndfile`, which does not support those formats.
</details>
<details>
  <summary>Could the player automatically obtain lyrics?</summary>
  Yes, it could, however it would need to know the Spotify track ID of the song.
</details>
<details>
  <summary>I hear "crackling" when playing FLAC files.</summary>
  
  This issue should now be fixed, however, if you're still experiencing it, check if you're using a debug build. If yes, such behavior can be expected. Please use release builds instead.
</details>
<details>
  <summary>Why is it taking so long to open the player? ("Launching..." takes long)</summary>
  
  The player is designed to initialize `rodio`, load the audio file and lyrics first, before creating and drawing the UI.
  I personally like having a longer startup time, rather than a "laggy-looking" UI.

  Also, __debug builds have significantly higher loading times__!
</details>