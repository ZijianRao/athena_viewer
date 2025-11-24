use std::io::{self};
use std::time::{Duration, Instant};

use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Paragraph},
    DefaultTerminal, Frame,
};
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

mod message_holder;

#[derive(Debug, Default)]
struct App {
    input_mode: InputMode,
    input: Input,
    message_holder: message_holder::message_holder::MessageHolder,
}

#[derive(Debug, Default, PartialEq)]
enum InputMode {
    #[default]
    Normal,
    FileSearch,
    FileView,
}

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal);
    ratatui::restore();
    app_result
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let tick_rate = Duration::from_millis(250);
        let mut last_tick = Instant::now();

        loop {
            terminal.draw(|frame| self.draw(frame))?;
            let timeout = tick_rate.saturating_sub(last_tick.elapsed());

            if event::poll(timeout)? {
                let event = event::read()?;
                if let Event::Key(key_event) = event {
                    match self.input_mode {
                        InputMode::FileView => match key_event.code {
                            KeyCode::Char('q') => {
                                self.message_holder.reset();
                                self.input_mode = InputMode::FileSearch;
                            }
                            KeyCode::Char('j') | KeyCode::Down => {
                                self.message_holder.vertical_scroll =
                                    self.message_holder.vertical_scroll.saturating_add(1).min(
                                        self.message_holder.file_text_info.as_ref().unwrap().n_rows,
                                    );
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
                                self.message_holder.vertical_scroll =
                                    self.message_holder.vertical_scroll.saturating_add(30).min(
                                        self.message_holder.file_text_info.as_ref().unwrap().n_rows,
                                    );
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
                        },
                        InputMode::Normal => match key_event.code {
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Tab => {
                                self.input_mode = InputMode::FileSearch;
                                self.message_holder.setup();
                            }
                            _ => {}
                        },
                        InputMode::FileSearch => match key_event.code {
                            KeyCode::Tab => self.input_mode = InputMode::Normal,
                            KeyCode::Enter => {
                                self.message_holder.submit();
                                if self.message_holder.file_opened.is_some() {
                                    self.input_mode = InputMode::FileView;
                                }
                                self.input.reset();
                            }
                            _ => {
                                self.input.handle_event(&event);
                                self.message_holder.update(self.input.value());
                            }
                        },
                    }
                }
            }
            if last_tick.elapsed() >= tick_rate {
                last_tick = Instant::now();
            }
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let vertical = Layout::vertical([
            Constraint::Min(1),
            Constraint::Length(3),
            Constraint::Length(1),
        ]);

        let [messages_area, input_area, help_area] = vertical.areas(frame.area());
        self.draw_help_area(help_area, frame);
        self.draw_input_area(input_area, frame);
        self.message_holder.draw(messages_area, frame);
    }

    fn draw_help_area(&self, area: Rect, frame: &mut Frame) {
        let instructions: Text;
        match self.input_mode {
            InputMode::Normal => {
                instructions = Text::from(Line::from(vec![
                    " Normal ".bold(),
                    " Switch Mode ".into(),
                    "<Tab>".blue().bold(),
                    " Quit ".into(),
                    "<Q>".blue().bold(),
                ]));
            }
            InputMode::FileSearch => {
                instructions = Text::from(Line::from(vec![
                    " FileSearch ".bold(),
                    " Switch Mode ".into(),
                    "<Tab>".blue().bold(),
                ]));
            }
            InputMode::FileView => {
                instructions = Text::from(Line::from(vec![
                    " FileView".bold(),
                    "Use h j k l or ◄ ▲ ▼ ► to scroll ".bold(),
                    " Quit ".into(),
                    "<Q>".blue().bold(),
                ]));
            }
        }
        let help_message = Paragraph::new(instructions);
        frame.render_widget(help_message, area);
    }

    fn draw_input_area(&self, area: Rect, frame: &mut Frame) {
        // keep 2 for boarders and 1 for cursor
        let width = area.width.max(3) - 3;
        let scroll = self.input.visual_scroll(width as usize);
        let style = match self.input_mode {
            InputMode::FileSearch => Color::Yellow.into(),
            _ => Style::default(),
        };

        let input = Paragraph::new(self.input.value())
            .style(style)
            .scroll((0, scroll as u16))
            .block(Block::bordered().title("Input"));
        frame.render_widget(input, area);

        // https://github.com/sayanarijit/tui-input/blob/main/examples/ratatui_crossterm_input.rs
        if self.input_mode == InputMode::FileSearch {
            let x = self.input.visual_cursor().max(scroll) - scroll + 1;
            frame.set_cursor_position((area.x + x as u16, area.y + 1));
        }
    }
}
