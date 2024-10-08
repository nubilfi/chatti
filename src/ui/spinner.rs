//! Provides a simple text-based spinner for indicating progress.

/// A simple text-based spinner for indicating progress
#[derive(Default, Debug)]
pub struct Spinner {
    frames: Vec<char>,
    current: usize,
}

impl Spinner {
    /// Creates a new Spinner instance.
    ///
    /// # Returns
    ///
    /// A new `Spinner` instance initialized with default spinner frames.
    ///
    /// # Examples
    ///
    /// ```
    /// use chatti::ui::spinner::Spinner;
    ///
    /// let spinner = Spinner::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Spinner {
            frames: vec!['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'],
            current: 0,
        }
    }

    /// Returns the next frame of the spinner.
    ///
    /// # Returns
    ///
    /// A `char` representing the next frame of the spinner.
    ///
    /// # Examples
    ///
    /// ```
    /// use chatti::ui::spinner::Spinner;
    ///
    /// let mut spinner = Spinner::new();
    /// let frame1 = spinner.next_frame();
    /// let frame2 = spinner.next_frame();
    /// assert_ne!(frame1, frame2);
    /// ```
    pub fn next_frame(&mut self) -> char {
        let char = self.frames[self.current];
        self.current = (self.current + 1) % self.frames.len();
        char
    }
}
