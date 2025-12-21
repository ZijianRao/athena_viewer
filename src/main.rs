use athena_viewer::app;
use std::io::{self};

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = app::App::new().run(&mut terminal);
    ratatui::restore();
    app_result
}
