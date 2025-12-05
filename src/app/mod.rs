use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Paragraph},
    Frame,
};
use std::io::{self};
use std::time::{Duration, Instant};

use ratatui::DefaultTerminal;
use tui_input::Input;

use crate::message_holder::message_holder::MessageHolder;
use crate::state_holder::state_holder::{InputMode, StateHolder, ViewMode};

#[derive(Debug, Default)]
pub struct App {
    state_holder: StateHolder,
    input: Input,
    exit: bool,
    message_holder: MessageHolder,
}

pub mod state_handler;
impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        use InputMode::*;
        use ViewMode::*;
        let tick_rate = Duration::from_millis(200);
        let mut last_tick = Instant::now();

        loop {
            terminal.draw(|frame| self.draw(frame))?;
            match (
                self.state_holder.input_mode.clone(),
                self.state_holder.view_mode.clone(),
            ) {
                (Normal, Search) => self.handle_normal_search_event(),
                (Normal, FileView) => {
                    self.handle_normal_file_view_event(&mut last_tick, &tick_rate)
                }
                (Edit, HistoryFolderView) => self.handle_edit_history_folder_view_event(),
                (Edit, Search) => self.handle_edit_search_event(),
                _ => (),
            }
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
        match (
            self.state_holder.input_mode.clone(),
            self.state_holder.view_mode.clone(),
        ) {
            (Normal, Search) => self.draw_help_normal_search(help_area, frame),
            (Normal, FileView) => self.draw_help_normal_file_view(help_area, frame),
            (Edit, HistoryFolderView) => self.draw_help_edit_history_folder_view(help_area, frame),
            (Edit, Search) => self.draw_edit_search(help_area, frame),
            _ => (),
        }
        // self.draw_help_area(help_area, frame);
        self.draw_input_area(input_area, frame);
        self.message_holder.draw(messages_area, frame);
    }

    pub fn draw_input_area(&self, area: Rect, frame: &mut Frame) {
        let is_file_search_mode = self.state_holder.view_mode == ViewMode::Search;
        // keep 2 for boarders and 1 for cursor
        let width = area.width.max(3) - 3;
        let scroll = self.input.visual_scroll(width as usize);

        let style;
        if is_file_search_mode {
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
        if is_file_search_mode {
            let x = self.input.visual_cursor().max(scroll) - scroll + 1;
            frame.set_cursor_position((area.x + x as u16, area.y + 1));
        }
    }
}
