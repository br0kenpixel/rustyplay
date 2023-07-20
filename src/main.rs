#![allow(
    clippy::similar_names,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::module_name_repetitions
)]

use std::env;
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;

mod audioinfo;
mod display;
mod lyrics;
mod lyrics_parse;
mod player;
mod scrolledbuf;
mod timer;

use crate::audioinfo::AudioFile;
use crate::display::{Display, DisplayEvent};
use crate::lyrics::LyricsProcessor;
use crate::player::Player;

/// A list of supported audio formats.
const SUPPORTED_FORMATS: [&str; 3] = ["wav", "flac", "ogg"];

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Invalid arguments:");
        eprintln!("Usage:\n {} [FILE]", args[0]);
        eprintln!(
            "Supported formats: {}",
            SUPPORTED_FORMATS.map(str::to_ascii_uppercase).join(", ")
        );
        exit(1);
    }

    println!("Launching...");
    run(&args[1]);
}

/// Runs the program.
fn run(file: &str) {
    /* Initialize everything first, so the UI doesn't appear laggy/frozen for too long */
    let afile = AudioFile::new(file);
    let player = Player::new(file);
    let lyrics = LyricsProcessor::load_file(generate_lyrics_file_name(file));
    let mut lyrics_bank = None;

    /* Start UI */
    let mut display = Display::new(file);

    display.init();

    if !display.sizecheck() {
        display.destroy();
        eprintln!("Terminal is too small!");
        eprintln!("The minimum required size is 100x28");
        exit(1);
    }

    display.set_track_info(&afile.metadata);
    display.set_track_length(afile.length);
    display.set_file_quality(&afile);

    if lyrics.is_err() {
        display.set_unavailable();
        display.refresh();
    }

    display.set_playback_status(true);
    player.play();

    while !player.is_finished() {
        if !player.is_paused() {
            display.update_progress(player.playtime(), afile.length);
            display.handle_scroll();

            if lyrics.is_ok() {
                // SAFETY: We just checked if `lyrics` is `Ok()`.
                let lp = unsafe { lyrics.as_ref().unwrap_unchecked() };
                let playtime = player.playtime();
                let mut bank = lyrics_bank.unwrap_or_else(|| lp.get_bank(None));

                if bank.is_expired(playtime) && bank.next_available() {
                    bank = lp.get_bank(Some(bank));
                }

                let active = bank.get_active(playtime);
                display.set_lyrics_bank(&bank);
                display.set_active_lyrics_line(&active);
                display.refresh_infoview();

                lyrics_bank = Some(bank);
            }
        }

        display.staus_message_tick();

        // Getch will also refresh the display
        display.capture_event().map_or((), |event| {
            process_display_event(event, &player, &mut display);
        });

        sleep(Duration::from_millis(10));
    }

    player.destroy();
    display.destroy();
}

/// Process the current [`DisplayEvent`](DisplayEvent).
fn process_display_event(event: DisplayEvent, player: &Player, display: &mut Display) {
    match event {
        DisplayEvent::MakePlay => {
            player.play();
            display.set_playback_status(true);
            display.set_status_message("Resumed");
        }
        DisplayEvent::MakePause => {
            player.pause();
            display.set_playback_status(false);
            display.set_status_message("Paused");
        }
        DisplayEvent::ToggleMute => {
            if player.is_muted() {
                player.unmute();
                display.set_status_message("Unmuted");
            } else {
                player.mute();
                display.set_status_message("Muted");
            }
        }
        DisplayEvent::JumpNext | DisplayEvent::JumpBack => (), //TODO: Implement
        DisplayEvent::VolUp => {
            player.inc_volume();
            display.set_status_message(&format!("+ Volume ({}%)", player.get_volume()));
        }
        DisplayEvent::VolDown => {
            player.dec_volume();
            display.set_status_message(&format!("- Volume ({}%)", player.get_volume()));
        }
        DisplayEvent::Invalid(c) => {
            if c.is_ascii_alphanumeric() {
                display.set_status_message(&format!("Unknown command '{c}'"));
            } else {
                display.set_status_message("Unknown command");
            }
        }
        DisplayEvent::Quit => player.destroy(),
    }
}

/// Generates a file name for the lyrics file.  
/// This just replaces the file extension with `.json`.
fn generate_lyrics_file_name(file: &str) -> String {
    let no_ext = &file[0..file.rfind('.').unwrap()];
    let mut result = String::from(no_ext);
    result.push_str(".json");

    result
}
