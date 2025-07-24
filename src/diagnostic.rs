use crate::lexer::Span;

#[derive(Clone)]
pub struct Diagnostic {
    message: String,
    span: Span,
    hint: Option<(String, Span)>,
}

const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";
const CYAN: &str = "\x1b[36m";
const BRIGHT_RED: &str = "\x1b[91m";

impl Diagnostic {
    pub fn report_error(message: String, span: Span) -> Diagnostic {
        Diagnostic {
            message,
            span,
            hint: None,
        }
    }
    pub fn report_error_with_hint(message: String, span: Span, hint: (String, Span)) -> Diagnostic {
        Diagnostic {
            message,
            span,
            hint: Some(hint),
        }
    }

    pub fn display_diagnostic(&self, filename: &str, source: &str) {
        let message = format!("{}{}error:{} {}", BOLD, RED, RESET, self.message);

        Self::display_message(filename, source, message, self.span);

        if let Some((message, span)) = &self.hint {
            let message = format!("{}hint:{} {}", YELLOW, RESET, message);
            Self::display_message(filename, source, message, *span);
        }
    }

    fn display_message(filename: &str, source: &str, message: String, span: Span) {
        let mut line_start = 0;

        for (line_index, line) in source.lines().enumerate() {
            let line_len = line.len();
            let line_end = line_start + line_len;

            if span.offset >= line_start && span.offset < line_end {
                let offset_in_line = span.offset - line_start;
                let marker_len = if offset_in_line + span.length > line_len {
                    line_len.saturating_sub(offset_in_line)
                } else {
                    span.length.max(1)
                };

                let location = format!("{}:{}:{}", filename, line_index + 1, offset_in_line + 1);

                let marker_line: String = " ".repeat(offset_in_line) + &"^".repeat(marker_len);

                eprintln!(
                    "{} {}{}{}\n {}|\t{}\n {}|\t{}{}{}",
                    message,
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
    }
}
