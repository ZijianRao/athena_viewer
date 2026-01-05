use ratatui::prelude::*;
use std::path::Path;
use syntect::{
    easy::HighlightLines,
    highlighting::{Theme, ThemeSet},
    parsing::{SyntaxReference, SyntaxSet},
    util::LinesWithEndings,
};

use crate::app::app_error::{AppError, AppResult};

/// Syntax highlighter for code files using syntect
///
/// # Fields
///
/// - `syntax_set`: Set of syntax definitions for various languages
/// - `theme`: Color theme for highlighting (base16-ocean.dark)
#[derive(Debug)]
pub struct CodeHighlighter {
    syntax_set: SyntaxSet,
    theme: Theme,
}

impl Default for CodeHighlighter {
    fn default() -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();
        let theme = theme_set.themes["base16-ocean.dark"].clone();

        Self { syntax_set, theme }
    }
}

impl CodeHighlighter {
    /// Gets the appropriate syntax for a file based on its extension
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to the file
    ///
    /// # Returns
    ///
    /// Returns the syntax reference for the file type, or plain text syntax
    /// as a fallback
    fn get_syntax(&self, file_path: &Path) -> &SyntaxReference {
        file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext_str| self.syntax_set.find_syntax_by_extension(ext_str))
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text())
    }

    /// Applies syntax highlighting to code
    ///
    /// # Arguments
    ///
    /// * `code` - The source code to highlight
    /// * `syntax` - The syntax definition to use
    ///
    /// # Returns
    ///
    /// Returns `AppResult<Vec<Line<'static>>>` which may contain `AppError::Parse`
    /// if highlighting fails
    fn get_highlighted_code(
        &self,
        code: &str,
        syntax: &SyntaxReference,
    ) -> AppResult<Vec<Line<'static>>> {
        let mut highlighter = HighlightLines::new(syntax, &self.theme);
        let mut lines = Vec::new();

        for line in LinesWithEndings::from(code) {
            let ranges = highlighter
                .highlight_line(line, &self.syntax_set)
                .map_err(|_| AppError::Parse("Unable to apply highlight for text file!".into()))?;
            let spans = ranges
                .into_iter()
                .map(|(style, text)| {
                    Span::styled(
                        text.to_string(),
                        Style::default().fg(Color::Rgb(
                            style.foreground.r,
                            style.foreground.g,
                            style.foreground.b,
                        )),
                    )
                })
                .collect::<Vec<_>>();
            lines.push(Line::from(spans));
        }
        Ok(lines)
    }

    /// Highlights a code string for a specific file
    ///
    /// # Arguments
    ///
    /// * `code` - The source code to highlight
    /// * `file_path` - Path to determine the language syntax
    ///
    /// # Returns
    ///
    /// Returns `AppResult<Vec<Line<'static>>>` which may contain:
    /// - `AppError::Parse`: If syntax highlighting fails
    pub fn highlight(&self, code: &str, file_path: &Path) -> AppResult<Vec<Line<'static>>> {
        let syntax = self.get_syntax(file_path);
        self.get_highlighted_code(code, syntax)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight_plain_test() {
        let highlighter = CodeHighlighter::default();
        let syntax = highlighter.syntax_set.find_syntax_plain_text();

        let code = "abc \n cde";

        let out = highlighter.get_highlighted_code(code, syntax);
        assert_eq!(out.unwrap().len(), 2)
    }
}
