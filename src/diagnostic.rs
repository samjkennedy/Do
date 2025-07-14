use crate::lexer::Span;

#[derive(Clone)]
pub struct Diagnostic {
    message: String,
    span: Span,
}

const RED: &str = "\x1b[31m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";
const CYAN: &str = "\x1b[36m";
const BRIGHT_RED: &str = "\x1b[91m";

impl Diagnostic {
    pub fn report_error(message: String, span: Span) -> Diagnostic {
        Diagnostic { message, span }
    }

    pub fn display_diagnostic(&self, filename: &str, source: &str) {
        let mut line_start = 0;

        for (line_index, line) in source.lines().enumerate() {
            let line_len = line.len();
            let line_end = line_start + line_len;

            if self.span.offset >= line_start && self.span.offset < line_end {
                let offset_in_line = self.span.offset - line_start;
                let marker_len = if offset_in_line + self.span.length > line_len {
                    line_len.saturating_sub(offset_in_line)
                } else {
                    self.span.length.max(1)
                };

                let location = format!("{}:{}:{}", filename, line_index + 1, offset_in_line + 1);

                let marker_line: String = " ".repeat(offset_in_line) + &"^".repeat(marker_len);

                eprintln!(
                    "{}{}error:{} {} {}{}{}\n {}|\t{}\n {}|\t{}{}{}",
                    BOLD,
                    RED,
                    RESET,
                    self.message,
                    CYAN,
                    location,
                    RESET,
                    line_index + 1,
                    line,
                    " ".repeat((line_index + 1).to_string().len()),
                    BRIGHT_RED,
                    marker_line,
                    RESET
                );
                return;
            }

            // Advance line_start to the beginning of the next line
            // +2 assumes Windows newlines (\r\n)
            line_start = line_end + 2; //TODO: handle newlines more gracefully
        }

        // fallback in case span doesn't match any line
        eprintln!(
            "error: {} (at byte offset {})",
            self.message, self.span.offset
        );
    }
}
