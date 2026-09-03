use crate::error::{GripError, ResultCategory};
use serde::Serialize;
use serde_json::{Map, Value};
use std::io::{self, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    Human,
    Json,
}

#[derive(Debug, Clone)]
pub struct CommandOutcome {
    pub category: ResultCategory,
    pub message: String,
    pub details: Map<String, Value>,
}

impl CommandOutcome {
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            category: ResultCategory::Success,
            message: message.into(),
            details: Map::new(),
        }
    }
    pub fn failure(error: &GripError) -> Self {
        Self {
            category: error.category(),
            message: error.to_string(),
            details: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ResultEnvelopeV1 {
    pub schema_version: u8,
    pub status: &'static str,
    pub code: &'static str,
    pub message: String,
    pub details: Map<String, Value>,
}

impl From<CommandOutcome> for ResultEnvelopeV1 {
    fn from(value: CommandOutcome) -> Self {
        Self {
            schema_version: 1,
            status: value.category.status(),
            code: value.category.code(),
            message: value.message,
            details: value.details,
        }
    }
}

pub fn render(outcome: CommandOutcome, mode: OutputMode, writer: &mut dyn Write) -> io::Result<()> {
    match mode {
        OutputMode::Human => writeln!(writer, "{}", outcome.message),
        OutputMode::Json => {
            serde_json::to_writer(&mut *writer, &ResultEnvelopeV1::from(outcome))?;
            writeln!(writer)
        }
    }
}

pub fn emit_diagnostic(verbosity: u8, event: &str, writer: &mut dyn Write) -> io::Result<()> {
    if verbosity > 0 {
        writeln!(writer, "diagnostic: {event}")
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn json_envelope_has_consistent_status_and_code() {
        let mut output = Vec::new();
        render(CommandOutcome::success("ok"), OutputMode::Json, &mut output).unwrap();
        let value: Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(value["status"], "ok");
        assert_eq!(value["code"], "ok");
    }

    struct FailingWriter;
    impl Write for FailingWriter {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("closed"))
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    #[test]
    fn renderer_propagates_output_failure() {
        assert!(
            render(
                CommandOutcome::success("ok"),
                OutputMode::Json,
                &mut FailingWriter
            )
            .is_err()
        );
    }
}
