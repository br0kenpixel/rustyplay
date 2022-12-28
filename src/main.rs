use std::env;
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;

mod audioinfo;
use crate::audioinfo::*;
mod display;
use crate::display::*;
mod player;
use crate::player::*;
mod lyrics;
use crate::lyrics::*;

/// A list of supported audio formats.
const SUPPORTED_FORMATS: [&str; 3] = ["wav", "flac", "ogg"];

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Invalid arguments:");
        eprintln!("Usage:\n {} [FILE]", args[0]);
        eprintln!("Supported formats: {:?}", SUPPORTED_FORMATS);
        exit(1);
    }

    println!("Launching...");
    run(args[1].clone());
}

/// Runs the program.
fn run(file: String) {
    /* Initialize everything first, so the UI doesn't appear laggy/frozen for too long */
    let afile: AudioFile = get_audio_info(&file);
    let player: Player = Player::new(&file);
    let lyrics = LyricsProcessor::load_file(generate_lyrics_file_name(&file));

    /* Start UI */
    let display: Display = Display::new();
    let mut display_event: DisplayEvent = DisplayEvent::Nothing;
    
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
    }

    display.set_playback_status(true);
    player.play();

    while !player.is_finished() {
        if !player.is_paused() {
            display.update_progress(player.playtime(), afile.length);

            if !lyrics.is_err() {
                let line = lyrics.as_ref().unwrap().get_line(player.playtime());
                if let Some(text) = line {
                    display.set_text(text);
                } else {
                    display.clear_infoview();
                }
                display.refresh_infoview();
            }
        }
        
        // Getch will also refresh the display
        match display.getch() {
            None => (),
            Some(key) => {
                display_event = match char::from_u32(key as u32).unwrap()  {
                    'g' => DisplayEvent::MakePlay,
                    'f' => DisplayEvent::JumpBack,
                    'h' => DisplayEvent::JumpNext,
                    'b' => DisplayEvent::MakePause,
                    'v' => DisplayEvent::ToggleMute,
                    'q' => DisplayEvent::Quit,
                    _   => DisplayEvent::Nothing
                }
            }
        }

        process_display_event(display_event, &player, &display);
        sleep(Duration::from_millis(10));
    }

    player.destroy();
    display.destroy();
}

/// Process the current [`DisplayEvent`](DisplayEvent).
fn process_display_event(event: DisplayEvent, player: &Player, display: &Display) {
    use DisplayEvent::*;

    match event {
        MakePlay => {
            player.play();
            display.set_playback_status(true);
        },
        MakePause => {
            player.pause();
            display.set_playback_status(false);
        },
        ToggleMute => {
            if player.is_muted() {
                player.unmute();
            } else {
                player.mute();
            }
        },
        JumpNext => (), //TODO: Implement
        JumpBack => (), //TODO: Implement
        Quit     => player.destroy(),
        Nothing  => ()
    }
}

/// Generates a file name for the lyrics file.  
/// This just replaces the file extension with `.json`.
fn generate_lyrics_file_name(file: &String) -> String {
    let file_str = file.as_str();
    let no_ext = &file_str[0..file_str.rfind('.').unwrap()];
    let mut result = String::from(no_ext);
    result.push_str(".json");

    result
}