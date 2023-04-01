use pausable_clock::PausableClock;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::fs::File;
use std::io::BufReader;
use std::time::{Duration, Instant};

/// This structure represents an audio player.
pub struct Player {
    /// *Unused but needs to be kept in memory.*
    _stream: OutputStream,
    /// *Unused but needs to be kept in memory.*
    _stream_handle: OutputStreamHandle,
    /// A "controller" kind of object.  
    /// It allows, for example, to pause the audio and resume it.
    sink: Sink,
    /// The time when the audio started playing.  
    /// *This is used to calculate the playtime*
    start_time: Instant,
    /// A clock that can be paused and resumed.  
    /// *This is used to calculate the playtime*  
    /// When the audio is paused, the clock is paused too.
    clock: PausableClock,
}

impl Player {
    /// Creates a new player from a given file.  
    /// *The playback is paused by default.*
    pub fn new(file: &String) -> Player {
        let (_stream, _stream_handle) =
            OutputStream::try_default().expect("Unable to open audio device");

        let sink = Sink::try_new(&_stream_handle).expect("Unable to create Sink");

        let file = BufReader::new(File::open(&file).expect("Unable to open file"));

        let source = Decoder::new(file).expect("Unable to create decoder");
        /* type: Decoder<BufReader<File>> */

        let start_time = Instant::now();
        let clock = PausableClock::default();

        // Start playling
        sink.append(source);
        sink.pause();
        clock.pause();

        Player {
            _stream,
            _stream_handle,
            sink,
            start_time,
            clock,
        }
    }

    /// Pauses the audio playback.
    pub fn pause(&self) {
        self.sink.pause();
        self.clock.pause();
    }

    /// Resumes the audio playback.
    pub fn play(&self) {
        self.sink.play();
        self.clock.resume();
    }

    /// Mutes the audio playback.
    pub fn mute(&self) {
        self.sink.set_volume(0.0);
    }

    /// Unmutes the audio playback.
    pub fn unmute(&self) {
        self.sink.set_volume(1.0);
    }

    /// Returns whether the audio playback is muted or not.
    pub fn is_muted(&self) -> bool {
        self.sink.volume() == 0.0
    }

    /// Returns whether the audio playback is paused or not.
    pub fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }

    /// Returns whether the audio playback is finished or not.
    pub fn is_finished(&self) -> bool {
        self.sink.empty()
    }

    /// Destroys the player.
    pub fn destroy(&self) {
        self.sink.stop();
    }

    /// Returns the current playtime.
    pub fn playtime(&self) -> Duration {
        Instant::from(self.clock.now()) - self.start_time
    }
}
