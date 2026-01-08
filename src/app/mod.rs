use ratatui::crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::DefaultTerminal;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Paragraph},
    Frame,
};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;
use std::time::Instant;
use tui_input::Input;

use crate::app::app_error::AppResult;
use crate::message_holder::MessageHolder;
use crate::state_holder::{InputMode, StateHolder, ViewMode};

/// Error types for the application
pub mod app_error;

/// State-specific event handlers for different input/view modes
///
/// Submodules:
/// - `normal_search` - Normal input mode with search view
/// - `normal_file_view` - Normal input mode with file viewing
/// - `edit_search` - Edit input mode with search view
/// - `edit_history_folder_view` - Edit input mode with history/folder view
pub mod state_handler;

const MIN_INPUT_WIDTH: u16 = 3;
const INPUT_WIDTH_PADDING: u16 = 3;
const TICK_RATE: Duration = Duration::from_millis(100);

/// Main application struct that manages the TUI state and rendering
///
/// # Fields
///
/// - `state_holder`: Shared state machine for input/view modes
/// - `input`: Current input buffer for search/filter operations
/// - `exit`: Flag to signal application termination
/// - `message_holder`: Manages file viewing, directory navigation, and display
/// - `timer`: Performance tracking timer
/// - `duration`: Elapsed time since last operation
/// - `log_message`: Current status/error message for display
#[derive(Debug)]
pub struct App {
    pub state_holder: Rc<RefCell<StateHolder>>,
    pub input: Input,
    pub exit: bool,
    pub message_holder: MessageHolder,
    pub timer: Instant,
    pub duration: Duration,
    pub log_message: String,
    state_changed: bool,
}

impl App {
    /// Creates a new application instance
    ///
    /// # Arguments
    ///
    /// * `current_directory` - The starting directory for file navigation
    ///
    /// # Returns
    ///
    /// Returns `AppResult<Self>` which may contain:
    /// - `AppError::Io`: If the directory cannot be read
    /// - `AppError::Path`: If path resolution fails
    /// - `AppError::Cache`: If initial cache setup fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::path::PathBuf;
    /// use athena_viewer::app::App;
    ///
    /// let app = App::new(PathBuf::from("/home/user")).unwrap();
    /// ```
    pub fn new(current_directory: PathBuf) -> app_error::AppResult<Self> {
        let state_holder = Rc::new(RefCell::new(StateHolder::default()));

        Ok(App {
            state_holder: Rc::clone(&state_holder),
            input: Input::default(),
            exit: false,
            message_holder: MessageHolder::new(current_directory, Rc::clone(&state_holder))?,
            timer: Instant::now(),
            duration: Duration::default(),
            log_message: "".into(),
            state_changed: true,
        })
    }
    /// Runs the main application loop
    ///
    /// This method handles the event loop, rendering, and error handling
    /// until the user exits (Ctrl+Z) or a terminal error occurs.
    ///
    /// # Arguments
    ///
    /// * `terminal` - The ratatui terminal to render to
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on clean exit or `AppError::Terminal` on terminal errors
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> app_error::AppResult<()> {
        loop {
            if self.state_changed {
                terminal.draw(|frame| self.draw(frame).expect("Unexpected!"))?;
                self.state_changed = false;
            }
            let result = self.handle_event();
            if let Err(err) = result {
                self.handle_error(err)
            }
            if self.exit {
                return Ok(());
            }
        }
    }
    /// Handles errors by updating the log message and potentially exiting
    ///
    /// # Arguments
    ///
    /// * `error` - The application error to handle
    fn handle_error(&mut self, error: app_error::AppError) {
        use app_error::AppError::*;
        self.log_message = error.to_string();
        if let Terminal(_) = error {
            self.exit = true;
        }
    }

    /// Renders the current application state to the terminal frame
    ///
    /// This method draws all UI components including:
    /// - Help text based on current mode
    /// - Input area
    /// - Main content (file view or directory listing)
    /// - Log area with timing and status info
    ///
    /// # Arguments
    ///
    /// * `frame` - The ratatui frame to render to
    ///
    /// # Returns
    ///
    /// Returns `AppResult<()>` which may contain `AppError::Terminal` on render errors
    pub fn draw(&mut self, frame: &mut Frame) -> AppResult<()> {
        use InputMode::*;
        use ViewMode::*;
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(3),
            Constraint::Length(1),
        ]);

        let [log_area, messages_area, input_area, help_area] = vertical.areas(frame.area());
        let input_mode = self.state_holder.borrow().input_mode;
        let view_mode = self.state_holder.borrow().view_mode;

        match (input_mode, view_mode) {
            (Normal, Search) => self.draw_help_normal_search(help_area, frame),
            (Normal, FileView) => self.draw_help_normal_file_view(help_area, frame),
            (Edit, HistoryFolderView) => self.draw_help_edit_history_folder_view(help_area, frame),
            (Edit, Search) => self.draw_edit_search(help_area, frame),
            _ => (),
        }
        self.draw_input_area(input_area, frame);
        self.message_holder.draw(messages_area, frame)?;
        self.draw_log_area(log_area, frame);

        Ok(())
    }

    /// Renders the input area with the current input buffer
    ///
    /// # Arguments
    ///
    /// * `area` - The rectangular area to render in
    /// * `frame` - The ratatui frame to render to
    pub fn draw_input_area(&self, area: Rect, frame: &mut Frame) {
        // keep 2 for boarders and 1 for cursor
        let width = area.width.max(MIN_INPUT_WIDTH) - INPUT_WIDTH_PADDING;
        let scroll = self.input.visual_scroll(width as usize);

        let style = if self.state_holder.borrow().is_edit() {
            Color::Yellow.into()
        } else {
            Style::default()
        };

        let input = Paragraph::new(self.input.value())
            .style(style)
            .scroll((0, scroll as u16))
            .block(Block::bordered().title("Input"));
        frame.render_widget(input, area);

        // https://github.com/sayanarijit/tui-input/blob/main/examples/ratatui_crossterm_input.rs
        if self.state_holder.borrow().is_edit() {
            let x = self.input.visual_cursor().max(scroll) - scroll + 1;
            frame.set_cursor_position((area.x + x as u16, area.y + 1));
        }
    }

    /// Renders the log area with timing and status information
    ///
    /// # Arguments
    ///
    /// * `area` - The rectangular area to render in
    /// * `frame` - The ratatui frame to render to
    pub fn draw_log_area(&self, area: Rect, frame: &mut Frame) {
        let log = Paragraph::new(format!("Took {:.2?} {}", self.duration, self.log_message));
        frame.render_widget(log, area);
    }

    /// Marks the start time for performance tracking
    pub fn mark_time(&mut self) {
        self.timer = Instant::now()
    }

    /// Calculates elapsed time since the last mark
    ///
    /// Updates the `duration` field with the time elapsed since `mark_time()` was called
    pub fn since_mark(&mut self) {
        self.duration = self.timer.elapsed()
    }

    /// Handles incoming terminal events
    ///
    /// This method polls for events and dispatches them to the appropriate
    /// state-specific handler based on the current input and view modes.
    ///
    /// # Returns
    ///
    /// Returns `AppResult<()>` which may contain:
    /// - `AppError::Terminal`: If event polling or reading fails
    /// - Handler-specific errors from state handlers
    pub fn handle_event(&mut self) -> app_error::AppResult<()> {
        use InputMode::*;
        use ViewMode::*;
        if event::poll(TICK_RATE)
            .map_err(|_| app_error::AppError::Terminal("Unable to pool".into()))?
        {
            let event = event::read()
                .map_err(|_| app_error::AppError::Terminal("Unable to parse event".into()))?;
            let mut is_key_press_event = false;

            if let Event::Key(key_event) = &event {
                self.mark_time();
                is_key_press_event = true;
                self.state_changed = true;
                if let &KeyCode::Char('z') = &key_event.code {
                    if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                        self.exit = true;
                    }
                };
            };
            if self.exit {
                return Ok(());
            }

            let input_mode = self.state_holder.borrow().input_mode;
            let view_mode = self.state_holder.borrow().view_mode;
            match (input_mode, view_mode) {
                (Normal, Search) => self.handle_normal_search_event(event)?,
                (Normal, FileView) => self.handle_normal_file_view_event(event)?,
                (Edit, HistoryFolderView) => self.handle_edit_history_folder_view_event(event)?,
                (Edit, Search) => self.handle_edit_search_event(event)?,
                _ => (),
            }

            if is_key_press_event {
                self.since_mark();
            }
        }
        Ok(())
    }
}
