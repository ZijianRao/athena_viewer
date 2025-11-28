use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};
use std::path::{Path, PathBuf};
use syntect::{
    easy::HighlightLines,
    highlighting::{Theme, ThemeSet},
    parsing::SyntaxSet,
    util::LinesWithEndings,
};

#[derive(Debug)]
pub struct CodeHighlighter {
    syntax_set: SyntaxSet,
    theme: Theme,
}

impl CodeHighlighter {
    pub fn new() -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();
        let theme = theme_set.themes["base16-ocean.dark"].clone();

        Self { syntax_set, theme }
    }

    pub fn highlight<'a>(&self, code: &'a str, file_path: &PathBuf) -> Vec<Line<'a>> {
        let syntax = self.syntax_set.find_syntax_by_extension("rs").unwrap();

        // let syntax = self
        //     .syntax_set
        //     .find_syntax_by_path(file_path.to_str().unwrap())
        //     .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        let mut highlighter = HighlightLines::new(syntax, &self.theme);
        let mut lines = Vec::new();

        for line in LinesWithEndings::from(code) {
            let ranges = highlighter.highlight_line(line, &self.syntax_set).unwrap();
            let spans = ranges
                .into_iter()
                .map(|(style, text)| {
                    Span::styled(
                        text,
                        Style::default().fg(Color::Rgb(
                            style.foreground.r,
                            style.foreground.g,
                            style.foreground.b,
                        )), // .bg(Color::Rgb(
                            //     style.background.r,
                            //     style.background.g,
                            //     style.background.b,
                            // )),
                    )
                })
                .collect::<Vec<_>>();
            lines.push(Line::from(spans));
        }

        lines
    }
}
