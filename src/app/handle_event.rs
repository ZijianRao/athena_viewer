use std::time::{Duration, Instant};

use ratatui::crossterm::event::{self, Event, KeyCode};
use tui_input::backend::crossterm::EventHandler;

use super::App;
use super::InputMode;

impl App {
    pub fn handle_file_view_event(&mut self, last_tick: &mut Instant, tick_rate: &Duration) {
        let timeout = tick_rate.saturating_sub(last_tick.elapsed());

        if event::poll(timeout).unwrap() {
            let event = event::read().unwrap();
            if let Event::Key(key_event) = event {
                match key_event.code {
                    KeyCode::Char('q') => {
                        self.message_holder.reset();
                        self.input_mode = InputMode::FileSearch;
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
    pub fn handle_normal_event(&mut self) {
        let event = event::read().unwrap();
        if let Event::Key(key_event) = event {
            match key_event.code {
                KeyCode::Char('q') => self.exit = true,
                KeyCode::Tab => {
                    self.input_mode = InputMode::FileSearch;
                    self.message_holder.setup();
                }
                _ => {}
            }
        }
    }

    pub fn handle_file_search_event(&mut self) {
        let event = event::read().unwrap();
        if let Event::Key(key_event) = event {
            match key_event.code {
                KeyCode::Tab => self.input_mode = InputMode::Normal,
                KeyCode::Up => {
                    self.message_holder.highlight_index =
                        self.message_holder.highlight_index.saturating_sub(1);
                }
                KeyCode::Down => {
                    self.message_holder.highlight_index =
                        self.message_holder.highlight_index.saturating_add(1);
                }
                KeyCode::Enter => {
                    self.message_holder.submit();
                    if self.message_holder.file_opened.is_some() {
                        self.input_mode = InputMode::FileView;
                    }
                    self.input.reset();
                }
                _ => {
                    self.input.handle_event(&event);
                    self.message_holder.highlight_index = 0;
                    self.message_holder.update(self.input.value());
                }
            }
        }
    }
}
