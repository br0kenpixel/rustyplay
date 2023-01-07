use ncurses::*;
use crate::audioinfo::{AudioMeta, AudioFile, AudioFormat};
use crate::scrolledbuf::*;
use crate::timer::Timer;
use std::path::Path;
use std::time::Duration;

/// Title string
const HEADER: &str = "[br0kenpixel's Music Player]";
/// Used to adjust the location of the `Lyrics` subwindow.
const INFOVIEW_OFFSET: i32 = 8;
/// Used to adjust the location of the status message.
const STATUSMSG_OFFSET: i32 = 6;
/// The default display time for a status message in seconds.
const STATUSMSG_DEFTIME: u64 = 2;
/// Amount of time to wait before scrolling the text in milliseconds.
const SCROLL_SHORT_TIME: u64 = 200;
/// Amount of time to wait before reversing the scroll direction.
const SCROLL_PAUSE_TIME: u64 = 3000;

/// Represents the terminal UI (TUI)
pub struct Display {
    /// Lyrics subwindow
    infoview: WINDOW,
    /// Scrollable text (used to scroll the file name across the UI)
    scrolledname: ScrolledBuf,
    /// Timer that handles scrolling
    scroll_timer: Timer,
    /// Timer that handles removing the status message after it's expired
    message_timer: Option<Timer>
}

/// Represents different events that occur when
/// using the keyboard controls.
#[derive(PartialEq, Clone, Copy)]
pub enum DisplayEvent {
    /// Nothing to do (no key was pressed)
    Nothing,
    /// The program was requested to resume playback.
    MakePlay,
    /// The program was requested to pause playback.
    MakePause,
    /// The program was requested to jump to the next track in the queue.
    JumpNext,
    /// The program was requested to jump to the previous track in the queue.
    JumpBack,
    /// The program was requested to mute or unmute the audio.
    ToggleMute,
    /// The user pressed a key which is not bound to any command.
    Invalid,
    /// The program was requested to stop playing and exit.
    Quit
}

/// This implementation contains all the functions that are used to draw the TUI.
impl Display {
    /// Creates the TUI and initializes [`ncurses`](ncurses).
    /// This function __does not__ draw the static components of the TUI.
    pub fn new(file: &String) -> Display {
        let locale_conf = LcCategory::all;
        setlocale(locale_conf, "en_US.UTF-8");

        initscr();
        noecho();
        timeout(0);
        curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

        let filename = 
            Path::new(file)
            .file_name()
            .unwrap()
            .to_owned()
            .into_string()
            .unwrap();

        Display {
            infoview: newwin(6, COLS() - 8, INFOVIEW_OFFSET, 4),
            scrolledname: ScrolledBuf::new(filename, COLS() - 8, ScrollDirection::LeftToRight),
            scroll_timer: Timer::new(Duration::from_millis(SCROLL_SHORT_TIME)),
            message_timer: None
        }
    }

    /// Checks if the terminal is big enough to display the TUI.
    /// A mininum size of 100x28 is required.  
    /// Sizes >= 100x28 will work and the TUI is adjusted automatically.
    pub fn sizecheck(&self) -> bool {
        return LINES() >= 28 && COLS() >= 100;
    }

    /// Initializes the TUI.
    /// For now this only calls [`Display::draw_ui()`](Self::draw_ui)
    pub fn init(&self) {
        self.draw_ui();
    }

    /// Draws the static parts of the TUI.
    /// For now this only draws the border and calls [`Display::set_header()`](Self::set_header)
    /// which handles the rest.
    fn draw_ui(&self) {
        border(0, 0, 0, 0, 0, 0, 0, 0);
        self.set_header();
    }

    /// Draws the rest of the TUI, such as:
    /// - The [`HEADER`](HEADER)
    /// - Keyboard shortcuts guide
    /// - `Lyrics` subwindow
    /// - Progress bar (static parts only - like the borders)
    /// - etc...
    fn set_header(&self) {
        let ypos: i32 = 0;
        let xpos: i32 = (COLS() / 2) - (HEADER.len() as i32 / 2);

        self.moveto(ypos, xpos);
        self.addstr(HEADER);

        self.moveto(LINES() - 4, 0);
        addch(ACS_LTEE()); // Pretty corners
        self.addnch(ACS_HLINE(), COLS() - 2);
        addch(ACS_RTEE());

        self.print_controls();
        self.print_progressui();
        self.print_trackinfoui();
        self.print_lyricsarea();
    }

    /// Draws the static parts of the `Lyrics` subwindow
    fn print_lyricsarea(&self) {
        self.refresh();
        box_(self.infoview, ACS_VLINE(), ACS_HLINE());
        touchwin(self.infoview);
        self.wmoveto(0, 2, self.infoview);
        self.waddstr("[ Lyrics ]", self.infoview);
        wrefresh(self.infoview);
    }

    /// Draws the static parts of the metadata display (`Track:`, `Album:`, `Artist(s):`)
    fn print_trackinfoui(&self) {
        self.moveto(2, 4);
        self.addstring(&format!("{:5}", "Track:"));
        self.moveto(3, 4);
        self.addstring(&format!("{:5}", "Album:"));
        self.moveto(4, 4);
        self.addstring(&format!("{:5}", "Artist(s):"));
    }

    /// Draws the static parts of the progress bar and timestamp indicators
    fn print_progressui(&self) {
        self.moveto(LINES() - 5, 0);
        addch(ACS_LTEE());
        addch(ACS_HLINE());
        self.addstr("[|>]");
        addch(ACS_HLINE()); addch(ACS_HLINE());
        self.addstr("[00:00]");
        addch(ACS_HLINE());
        self.addchar('[');
        self.moveto(LINES() - 5, COLS() - 11);
        self.addchar(']');
        addch(ACS_HLINE());
        self.addstr("[00:00]");
        addch(ACS_HLINE());
        addch(ACS_RTEE());
    }

    /// Draws the keyboard shortcuts guide
    fn print_controls(&self) {
        const EXIT_CTL_TXT: &str = "[Q] Exit";

        self.moveto(LINES() - 3, 2);
        //self.print_control('F', "Prev", true); // not implemented for now
        self.print_control('G', "Play", true);
        //self.print_control('H', "Next", false); // not implemented for now
        
        //self.moveto(LINES() - 2, 2);
        self.print_control('B', "Pause", true);
        self.print_control('V', "Mute", false);

        self.moveto(LINES() - 2, COLS() - 2 - EXIT_CTL_TXT.len() as i32);
        self.addstr(EXIT_CTL_TXT);
    }

    /// Draws a single keyboard shortcut guide
    fn print_control(&self, ctl_symbol: char, desc: &str, _continue: bool) {
        self.addstring(&format!("[{ctl_symbol}] {desc}"));
        if _continue {
            self.addchar(' ');
            addch(ACS_VLINE());
            self.addchar(' ');
        }
    }

    /// Refreshes the TUI by applying any changes done before calling this function.
    pub fn refresh(&self) {
        refresh();
        wrefresh(self.infoview);
    }

    /// Destroys the `Lyrics` subwindow and the main one.  
    /// Should be called when the player want's to exit.
    pub fn destroy(&self) {
        delwin(self.infoview);
        endwin();
    }

    /// A customized version of [`ncurses::getch()`](ncurses::getch()).  
    /// Returns an `Option<i32>` instead of returning an `i32` directly.  
    ///   
    /// If [`ncurses::getch()`](ncurses::getch()) returns [`ERR`(ERR)],
    /// `None` is returned instead.
    pub fn getch(&self) -> Option<i32> {
        match getch() {
            ERR => None,
            c => Some(c)
        }
    }

    /// __This is for debugging purposes only.__  
    /// A blocking version of [`getch()`](Self::getch()).  
    /// This may be useful since [`Display::new()`](Self::new()) enables non-blocking mode
    /// to prevent the player from freezing when checking for input.
    #[allow(dead_code)]
    pub fn blocking_getch(&self) -> i32 {
        let mut res = self.getch();
        while res == None {
            res = self.getch();
        }
        res.unwrap()
    }

    /// Alias for [`Display::waddchar()`](Self::waddchar()) with [`stdscr()`](ncurses::stdscr()) as the `win` argument.
    fn addchar(&self, c: char) {
        self.waddchar(c, stdscr());
    }

    /// Alias for [`ncurses::waddch()`](ncurses::waddch()) with `c` as a `char` instead of a `u32`/[`chtype`](chtype).
    fn waddchar(&self, c: char, win: WINDOW) {
        waddch(win, c as u32);
    }

    /// Alias for printring a character multiple times with [`ncurses::addch()`](ncurses::addch()).
    fn addnch(&self, c: chtype, n: i32) {
        for _ in 0..n {
            addch(c);
        }
    }

    /// Alias for [`Display::waddstr()`](Self::waddstr()) with [`stdscr()`](ncurses::stdscr()) as the `win` argument.
    fn addstr(&self, text: &str) {
        self.waddstr(text, stdscr());
    }

    /// Alias for [`ncurses::waddstr()`](ncurses::waddstr()).
    fn waddstr(&self, text: &str, win: WINDOW) {
        waddstr(win, text);
    }

    /// Alias for [`Display::waddstring()`](Self::waddstring()) with [`stdscr()`](ncurses::stdscr()) as the `win` argument.
    fn addstring(&self, text: &String) {
        self.waddstring(text, stdscr());
    }

    /// Alias for [`Display::waddstr()`](Self::waddstr()) but takes a [`&String`](String) instead of a [`&str`](str).
    fn waddstring(&self, text: &String, win: WINDOW) {
        self.waddstr(text.as_str(), win);
    }

    /// Alias for [`ncurses::addstr()`](ncurses::addstr()) but takes a [`u32`](u32) so it can print Unicode characters.  
    /// *This is used to draw the progressbar "blocks"*
    fn addwchar(&self, c: u32) {
        addstr(format!("{}", char::from_u32(c).unwrap()).as_ref());
    }

    /// Alias for [`ncurses::wmove()`](ncurses::wmove()) with a check to prevent the cursor from moving outside the screen.
    /// 
    /// ## Panics
    /// Panics if `ypos` or `xpos` is greater than the screen size.
    fn wmoveto(&self, ypos: i32, xpos: i32, win: WINDOW) {
        if ypos >= LINES() || xpos >= COLS() {
            panic!("moveto(ypos={ypos}, xpos={xpos}) exceeds screen size {}Lx{}C",
                LINES(), COLS());
        }
        wmove(win, ypos, xpos);
    }

    /// Alias for [`Display::wmoveto()`](Self::wmoveto()) with [`stdscr()`](ncurses::stdscr()) as the `win` argument.
    fn moveto(&self, ypos: i32, xpos: i32) {
        self.wmoveto(ypos, xpos, stdscr());
    }
}

/// This implementation adds functions used to change dynamic parts of the TUI.
impl Display {
    /// The the playback status (playing/paused) indicator in the TUI.
    pub fn set_playback_status(&self, playing: bool) {
        self.moveto(LINES() - 5, 3);
        self.addstring(&String::from({
            if playing {
                "||"
            } else {
                "|>"
            }
        }));
    }

    /// Set the metadata display in the TUI.  
    /// This updates the `Title`, `Album` and `Artist` fields.
    pub fn set_track_info(&self, metadata: &AudioMeta) {
        self.moveto(2, 15);
        self.addstring(&metadata.title);
        self.moveto(3, 15);
        self.addstring(&metadata.album);
        self.moveto(4, 15);
        self.addstring(&metadata.artist);
    }
    
    /// Set the track length display in the TUI.
    pub fn set_track_length(&self, time: f64) {
        self.print_pretty_time(LINES() - 5, COLS() - 8, time);
    }

    /// Update the current playback time and progress bar in the TUI.  
    /// If you're looking for the progress bar implementation, check [`Display::set_progress()`](Self::set_progress()).
    pub fn update_progress(&self, time: Duration, total_len: f64) {
        self.set_playtime(time);
        self.set_progress(time.as_secs_f64(), total_len);
    }

    /// Set the current playback time in the TUI.
    fn set_playtime(&self, time: Duration) {
        self.print_pretty_time(LINES() - 5, 9, time.as_secs_f64());
    }

    /// Calculate the progress bar blocks and print them to the TUI.
    pub fn set_progress(&self, played: f64, total_len: f64) {
        let max_block_count = ((COLS() - 12) - 15) - 1;
        let mut use_blocks = Display::map(
            played,
            0.0,
            total_len,
            0.0,
            max_block_count as f64
        ) as i32;

        // Constrain
        if use_blocks < 0 {
            use_blocks = 0;
        } else if use_blocks > max_block_count {
            use_blocks = max_block_count;
        }

        self.print_progress_blocks(use_blocks as i32, max_block_count);
    }

    /// Update the file quality display in the TUI.
    pub fn set_file_quality(&self, fileinfo: &AudioFile) {
        self.moveto(6, 4);
        self.addstring(&format!("{} Hz, {}, {} {}",
            fileinfo.sample_rate,
            match fileinfo.stereo {
                true => "Stereo",
                false => "Mono"
            },
            match fileinfo.lossless {
                true => "Lossless",
                false => "Lossy"
            },
            match fileinfo.format {
                AudioFormat::FLAC => "FLAC",
                AudioFormat::WAV  => "WAV",
                AudioFormat::OGG  => "OGG"
            }
        ));
    }

    /// Update the progress bar in the TUI.  
    /// Unicode character 0x2587 is used as the "block" character.
    fn print_progress_blocks(&self, count: i32, total_space: i32) {
        self.moveto(LINES() - 5, 17);
        for _ in 0..count {
            self.addwchar(0x2587u32);
        }
        for _ in count..total_space {
            self.addchar(' ');
        }
    }

    /// Arduino's [`map()`](https://www.arduino.cc/reference/en/language/functions/math/map/) function.  
    /// Maps a value from one range to another.
    fn map(x: f64, in_min: f64, in_max: f64, out_min: f64, out_max: f64) -> f64 {
        (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min
    }

    /// Print a time in the format `mm:ss` to the TUI.
    fn print_pretty_time(&self, ypos: i32, xpos: i32, seconds: f64) {
        self.moveto(ypos, xpos);
        self.addstring(&format!(
            "{:02}:{:02}",
            (seconds / 60.0) as i32,
            (seconds % 60.0) as i32
        ));
    }

    /// Displays a message on the bottom of the screen for a given amount of time.
    /// If there is another message being displayed, it will be cleared.
    /// If `time` is not specified (set to `None`), [`STATUSMSG_DEFTIME`](STATUSMSG_DEFTIME)
    /// is used as a default value.
    pub fn set_status_message(&mut self, message: &str, time: Option<Duration>) {
        let message = format!("[ {message} ]");
        let xpos = (COLS() / 2) - (message.len() as i32 / 2);

        if !self.message_timer.is_none() {
            self.clear_status_message();
        }
        
        self.moveto(LINES() - STATUSMSG_OFFSET, xpos);
        attr_on(A_STANDOUT());
        self.addstring(&message);
        attr_off(A_STANDOUT());
        self.message_timer = Some(Timer::new(time.unwrap_or(Duration::from_secs(STATUSMSG_DEFTIME))));
    }

    /// Clears the currently displayed status message.
    ///  
    /// ## Note
    /// Can be safely called even if there is no status message being
    /// displayed, as in such cases the function will not do anything.
    pub fn clear_status_message(&mut self) {
        if self.message_timer.is_none() {
            return;
        }
        self.message_timer = None;
        self.moveto(LINES() - STATUSMSG_OFFSET, 1);
        self.addnch(' ' as u32, COLS() - 4);
    }

    /// Checks if the currently displayed status message
    /// expired. If yes, it will be cleared, otherwise nothing will be done.
    /// 
    /// ## Note #1
    /// Can be safely called even if there is no status message being
    /// displayed, as in such cases the function will not do anything.
    /// ## Note #2
    /// For good accuracy, this function should be called as often as possible.
    pub fn staus_message_tick(&mut self) {
        if let Some(timer) = &self.message_timer {
            if timer.expired() {
                self.clear_status_message();
            }
        }
    }

    /// Handles scrolling the file name
    /// This function should be called as often as possible
    /// for accurately timed scrolling.
    pub fn handle_scroll(&mut self) {
        if !self.scroll_timer.expired() { return; }
        self.moveto(INFOVIEW_OFFSET + 7, 4);
        self.addstr(&self.scrolledname.current_frame());
        if self.scrolledname.is_finished() {
            self.scrolledname.swap_direction();
            self.scroll_timer.rebuild(Duration::from_millis(SCROLL_PAUSE_TIME));
        } else {
            self.scroll_timer.rebuild(Duration::from_millis(SCROLL_SHORT_TIME));
        }
        self.scrolledname.next_frame();
    }
}

/// This implementation adds functions to use the `Lyrics` subwindow.
impl Display {
    /// Set the text in the `Lyrics` subwindow.
    /// > **Note:** This still needs work - like a proper line wrapping algorithm.
    pub fn set_text(&self, line: String) {
        assert!((line.len() as i32) < COLS() - 12 /* some random bound */);
        self.clear_infoview();
        if line.is_empty() { return; }
        self.wmoveto(1, 2, self.infoview);
        wattron(self.infoview, A_BOLD());
        self.waddstr("-> ", self.infoview);
        self.waddstring(&line, self.infoview);
        wattroff(self.infoview, A_BOLD());
    }

    /// Clear all text inside the `Lyrics` subwindow.
    pub fn clear_infoview(&self) {
        for ypos in 1..3 {
            for xpos in 2..COLS() - 10 {
                self.wmoveto(ypos, xpos, self.infoview);
                self.waddchar(' ', self.infoview);
            }
        }
    }

    pub fn refresh_infoview(&self) {
        wrefresh(self.infoview);
    }

    /// Set the `Lyrics` subwindow to display the "Unavailable" message.
    pub fn set_unavailable(&self) {
        self.clear_infoview();
        self.wmoveto(1, 2, self.infoview);
        wattron(self.infoview, A_ITALIC());
        self.waddstr("Unavailable", self.infoview);
        wattroff(self.infoview, A_ITALIC());
    }
}