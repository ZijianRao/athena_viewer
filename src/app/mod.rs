use ratatui::crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::DefaultTerminal;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Paragraph},
    Frame,
};
use std::cell::RefCell;
use std::io::{self};
use std::rc::Rc;
use std::time::Duration;
use tui_input::Input;

use crate::message_holder::message_holder::MessageHolder;
use crate::state_holder::state_holder::{InputMode, StateHolder, ViewMode};

const MIN_INPUT_WIDTH: u16 = 3;
const INPUT_WIDTH_PADDING: u16 = 3;
const TICK_RATE: Duration = Duration::from_millis(200);

#[derive(Debug)]
pub struct App {
    state_holder: Rc<RefCell<StateHolder>>,
    input: Input,
    exit: bool,
    message_holder: MessageHolder,
}

pub mod state_handler;
impl App {
    pub fn new() -> Self {
        let state_holder = Rc::new(RefCell::new(StateHolder::default()));

        App {
            state_holder: Rc::clone(&state_holder),
            input: Input::default(),
            exit: false,
            message_holder: MessageHolder::new(Rc::clone(&state_holder)),
        }
    }
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_event();
            if self.exit {
                return Ok(());
            }
        }
    }
    pub fn draw(&mut self, frame: &mut Frame) {
        use InputMode::*;
        use ViewMode::*;
        let vertical = Layout::vertical([
            Constraint::Min(1),
            Constraint::Length(3),
            Constraint::Length(1),
        ]);

        let [messages_area, input_area, help_area] = vertical.areas(frame.area());
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
        self.message_holder.draw(messages_area, frame);
    }

    pub fn draw_input_area(&self, area: Rect, frame: &mut Frame) {
        // keep 2 for boarders and 1 for cursor
        let width = area.width.max(MIN_INPUT_WIDTH) - INPUT_WIDTH_PADDING;
        let scroll = self.input.visual_scroll(width as usize);

        let style;
        if self.state_holder.borrow().is_edit() {
            style = Color::Yellow.into();
        } else {
            style = Style::default();
        }

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

    pub fn handle_event(&mut self) {
        use InputMode::*;
        use ViewMode::*;
        if event::poll(TICK_RATE).expect("Unable handle the timeout applied!") {
            let event = event::read().expect("Unable to handle key press event!");

            if let Event::Key(key_event) = &event {
                match &key_event.code {
                    &KeyCode::Char('c') | &KeyCode::Char('z') => {
                        if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                            self.exit = true;
                        }
                    }
                    _ => (),
                }
            }
            if self.exit {
                return;
            }

            let input_mode = self.state_holder.borrow().input_mode;
            let view_mode = self.state_holder.borrow().view_mode;
            match (input_mode, view_mode) {
                (Normal, Search) => self.handle_normal_search_event(event),
                (Normal, FileView) => self.handle_normal_file_view_event(event),
                (Edit, HistoryFolderView) => self.handle_edit_history_folder_view_event(event),
                (Edit, Search) => self.handle_edit_search_event(event),
                _ => (),
            }
        }
    }
}
