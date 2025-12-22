use athena_viewer::app;
use std::env;
use std::io::{self};

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let current_directory = env::current_dir().expect("Unable to get current directory!");
    let app_result = app::App::new(current_directory).run(&mut terminal);
    ratatui::restore();
    app_result
}
