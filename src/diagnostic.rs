use crate::lexer::Span;

pub struct Diagnostic {
    message: String,
    span: Span,
}

impl Diagnostic {
    pub fn report_error(message: String, span: Span) -> Diagnostic {
        Diagnostic { message, span }
    }

    pub fn display_diagnostic(&self, filename: &String) {
        eprintln!(
            "ERROR: {} at {} offset: {} length: {}",
            self.message, filename, self.span.offset, self.span.length
        );
    }
}
