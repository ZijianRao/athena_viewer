use ratatui::crossterm::event::{self, Event, KeyCode};
use ratatui::{
    layout::Rect,
    style::Stylize,
    text::{Line, Text},
    widgets::Paragraph,
    Frame,
};
use std::time::{Duration, Instant};

use crate::app::App;

impl App {
    pub fn handle_normal_file_view_event(&mut self, last_tick: &mut Instant, tick_rate: &Duration) {
        let timeout = tick_rate.saturating_sub(last_tick.elapsed());

        if event::poll(timeout).unwrap() {
            let event = event::read().unwrap();
            if let Event::Key(key_event) = event {
                match key_event.code {
                    KeyCode::Char('q') => {
                        self.message_holder.reset();
                        self.state_holder.borrow_mut().restore_previous_state();
                    }
                    KeyCode::Char('j') | KeyCode::Down => {
                        self.message_holder.vertical_scroll = self
                            .message_holder
                            .vertical_scroll
                            .saturating_add(1)
                            .min(self.message_holder.file_text_info.as_ref().unwrap().n_rows);
                        self.message_holder.vertical_scroll_state = self
                            .message_holder
                            .vertical_scroll_state
                            .position(self.message_holder.vertical_scroll);
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        self.message_holder.vertical_scroll =
                            self.message_holder.vertical_scroll.saturating_sub(1);
                        self.message_holder.vertical_scroll_state = self
                            .message_holder
                            .vertical_scroll_state
                            .position(self.message_holder.vertical_scroll);
                    }
                    KeyCode::Char('h') | KeyCode::Left => {
                        self.message_holder.horizontal_scroll =
                            self.message_holder.horizontal_scroll.saturating_sub(1);
                        self.message_holder.horizontal_scroll_state = self
                            .message_holder
                            .horizontal_scroll_state
                            .position(self.message_holder.horizontal_scroll);
                    }
                    KeyCode::Char('l') | KeyCode::Right => {
                        self.message_holder.horizontal_scroll =
                            self.message_holder.horizontal_scroll.saturating_add(1).min(
                                self.message_holder
                                    .file_text_info
                                    .as_ref()
                                    .unwrap()
                                    .max_line_length,
                            );
                        self.message_holder.horizontal_scroll_state = self
                            .message_holder
                            .horizontal_scroll_state
                            .position(self.message_holder.horizontal_scroll);
                    }
                    KeyCode::Home => {
                        self.message_holder.horizontal_scroll = 0;
                        self.message_holder.horizontal_scroll_state = self
                            .message_holder
                            .horizontal_scroll_state
                            .position(self.message_holder.horizontal_scroll);
                        self.message_holder.vertical_scroll = 0;
                        self.message_holder.vertical_scroll_state = self
                            .message_holder
                            .vertical_scroll_state
                            .position(self.message_holder.vertical_scroll);
                    }
                    KeyCode::End => {
                        self.message_holder.vertical_scroll = self
                            .message_holder
                            .file_text_info
                            .as_ref()
                            .unwrap()
                            .n_rows
                            .saturating_sub(30);
                        self.message_holder.vertical_scroll_state = self
                            .message_holder
                            .vertical_scroll_state
                            .position(self.message_holder.vertical_scroll);
                    }
                    KeyCode::PageDown => {
                        self.message_holder.vertical_scroll = self
                            .message_holder
                            .vertical_scroll
                            .saturating_add(30)
                            .min(self.message_holder.file_text_info.as_ref().unwrap().n_rows);
                        self.message_holder.vertical_scroll_state = self
                            .message_holder
                            .vertical_scroll_state
                            .position(self.message_holder.vertical_scroll);
                    }
                    KeyCode::PageUp => {
                        self.message_holder.vertical_scroll =
                            self.message_holder.vertical_scroll.saturating_sub(30);
                        self.message_holder.vertical_scroll_state = self
                            .message_holder
                            .vertical_scroll_state
                            .position(self.message_holder.vertical_scroll);
                    }
                    _ => (),
                }
            }
        }
        if last_tick.elapsed() >= *tick_rate {
            *last_tick = Instant::now();
        }
    }
    pub fn draw_help_normal_file_view(&mut self, help_area: Rect, frame: &mut Frame) {
        let instructions = Text::from(Line::from(vec![
            "FileView ".bold(),
            " Quit ".into(),
            "<Q>".light_blue().bold(),
        ]));
        let help_message = Paragraph::new(instructions);
        frame.render_widget(help_message, help_area);
    }
}
