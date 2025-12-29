use athena_viewer::app;
use std::env;

fn main() -> app::app_error::AppResult<()> {
    let mut terminal = ratatui::init();

    let current_directory = env::current_dir().map_err(|_| {
        app::app_error::AppError::Path("Unable to get current working directory".into())
    })?;
    let app_result = app::App::new(current_directory)?.run(&mut terminal);
    ratatui::restore();
    app_result
}
