/// Represents scrollable text.
///
/// Example:
/// ```rust
/// let mut text = ScrolledBuf::new("Hello, world!", 6, ScrollDirection::LeftToRight);
/// assert_eq!(text.current_frame(), String::from("      "));
/// text.next_frame();
/// assert_eq!(text.current_frame(), String::from("!     "));
/// text.next_frame();
/// assert_eq!(text.current_frame(), String::from("d!    "));
/// text.next_frame();
/// assert_eq!(text.current_frame(), String::from("ld!   "));
/// ```
pub struct ScrolledBuf {
    /// Text to be scrolled
    text: String,
    /// A counter to represent the internal state
    step: isize,
    /// Scroll direction
    dir: ScrollDirection,
    /// Amount of visible characters
    visible_len: isize,
}

/// Scroll direction
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ScrollDirection {
    /// Scroll text from right to left
    RightToLeft,
    /// Scroll text from left to right
    LeftToRight,
}

impl ScrolledBuf {
    /// Creates a new scrollable text object.
    ///
    /// # Arguments
    /// * `text` - Any object that can be converted to a [`String`](String) using `into()`
    /// * `visible` - Amount of visible characters
    /// * `dir` - Scroll direction
    ///
    /// ### Notes
    /// `visible` is converted to [`isize`](isize) internally.
    ///
    /// Example:
    /// ```rust
    /// let mut text = ScrolledBuf::new("Hello, world!", 6, ScrollDirection::RightToLeft);
    /// let mut text = ScrolledBuf::new(String::from("Hello, world!"), 6, ScrollDirection::LeftToRight);
    /// ```
    pub fn new<S: Into<String>>(text: S, visible: i32, dir: ScrollDirection) -> Self {
        let text = text.into();
        let visible = visible as isize;
        let step = match dir {
            ScrollDirection::RightToLeft => -visible,
            ScrollDirection::LeftToRight => text.len() as isize,
        };

        Self {
            text,
            step,
            dir,
            visible_len: visible,
        }
    }

    /// Return the current state of the buffer.
    /// The length of the returned string is always [`visible_len`](Self::visible_len).
    pub fn current_frame(&self) -> String {
        let mut result = String::new();

        let start = self.step;
        let end = start + self.visible_len;

        for i in start..end {
            result.push(self.text.chars().nth(i as usize).unwrap_or(' '));
        }

        result
    }

    /// Move to the next frame.  
    /// *(Scrolls the text by one step.)*
    ///
    /// ### Note #1
    /// [`is_finished()`](Self::is_finished()) should be called to check
    /// if this function is safe to call.
    /// ### Note #2
    /// Calling this function after [`is_finished()`](Self::is_finished())
    /// returns `true` will mess up the scroll effect.
    pub fn next_frame(&mut self) {
        match self.dir {
            ScrollDirection::RightToLeft => self.step += 1,
            ScrollDirection::LeftToRight => self.step -= 1,
        }
    }

    /// Move back to the previous frame.
    /// *(Scrolls the text by one step in the opposite direction.)*
    #[allow(dead_code)]
    pub fn previous_frame(&mut self) {
        match self.dir {
            ScrollDirection::RightToLeft => self.step -= 1,
            ScrollDirection::LeftToRight => self.step += 1,
        }
    }

    /// Reset the internal step counter to it's initial value.  
    /// *(Restarts the scrolling effect)*
    pub fn reset(&mut self) {
        self.step = match self.dir {
            ScrollDirection::RightToLeft => -self.visible_len,
            ScrollDirection::LeftToRight => self.text.len() as isize,
        };
    }

    /// Swap the direction of scrolling to the opposite direction.
    pub fn swap_direction(&mut self) {
        self.dir = match self.dir {
            ScrollDirection::RightToLeft => ScrollDirection::LeftToRight,
            ScrollDirection::LeftToRight => ScrollDirection::RightToLeft,
        };
        self.reset();
    }

    /// Returns whether the scrolling effect is finished.  
    /// [`next_frame()`](Self::next_frame()) should __not__ be called
    /// if this function returns `true`.
    pub fn is_finished(&self) -> bool {
        match self.dir {
            ScrollDirection::RightToLeft => self.step == self.text.len() as isize + 1,
            ScrollDirection::LeftToRight => self.step == -self.visible_len - 1,
        }
    }
}
