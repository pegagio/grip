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
        let mut outcome = Self {
            category: error.category(),
            message: error.to_string(),
            details: Map::new(),
        };
        if let GripError::Mapping {
            operation,
            reason,
            paths,
            kind,
            publication_visible,
            conflicts,
            ..
        } = error
        {
            outcome
                .details
                .insert("operation".into(), operation.clone().into());
            outcome
                .details
                .insert("reason".into(), reason.clone().into());
            if !paths.is_empty() {
                outcome.details.insert("paths".into(), paths.clone().into());
            }
            if let Some(kind) = kind {
                outcome.details.insert(
                    "kind".into(),
                    serde_json::to_value(kind).expect("mapping kind serializes"),
                );
            }
            if *publication_visible {
                outcome
                    .details
                    .insert("publication_visible".into(), true.into());
                outcome
                    .details
                    .insert("durability_confirmed".into(), false.into());
            }
            if !conflicts.is_empty() {
                outcome.details.insert(
                    "conflicts".into(),
                    serde_json::to_value(conflicts).expect("ownership conflicts serialize"),
                );
            }
        }
        outcome
    }

    pub fn mapping_success(
        operation: &str,
        message: &str,
        mapping: &crate::mapping::Mapping,
    ) -> Self {
        let mut outcome = Self::success(message);
        outcome.details.insert("operation".into(), operation.into());
        outcome.details.insert(
            "mapping".into(),
            serde_json::to_value(mapping).expect("mapping serializes"),
        );
        outcome
    }

    pub fn mapping_list(mappings: &[crate::mapping::Mapping]) -> Self {
        let mut outcome = Self::success(format!("{} mapping(s)", mappings.len()));
        outcome
            .details
            .insert("operation".into(), "mapping_list".into());
        outcome.details.insert(
            "mappings".into(),
            serde_json::to_value(mappings).expect("mappings serialize"),
        );
        outcome
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
        OutputMode::Human => {
            writeln!(writer, "{}", outcome.message)?;
            if let Some(conflicts) = outcome.details.get("conflicts").and_then(Value::as_array) {
                for conflict in conflicts {
                    writeln!(
                        writer,
                        "{}: {} {} {}",
                        conflict
                            .get("reason")
                            .and_then(Value::as_str)
                            .unwrap_or("ownership_conflict"),
                        conflict
                            .get("first_path")
                            .and_then(Value::as_str)
                            .unwrap_or("unknown"),
                        conflict
                            .get("relation")
                            .and_then(Value::as_str)
                            .unwrap_or("conflicts_with"),
                        conflict
                            .get("second_path")
                            .and_then(Value::as_str)
                            .unwrap_or("unknown")
                    )?;
                }
            }
            if let Some(mapping) = outcome.details.get("mapping") {
                render_human_mapping(mapping, writer)?;
            }
            if let Some(mappings) = outcome.details.get("mappings").and_then(Value::as_array) {
                for mapping in mappings {
                    render_human_mapping(mapping, writer)?;
                }
            }
            Ok(())
        }
        OutputMode::Json => {
            serde_json::to_writer(&mut *writer, &ResultEnvelopeV1::from(outcome))?;
            writeln!(writer)
        }
    }
}

fn render_human_mapping(mapping: &Value, writer: &mut dyn Write) -> io::Result<()> {
    writeln!(
        writer,
        "{} {} -> {}",
        mapping
            .get("kind")
            .and_then(Value::as_str)
            .unwrap_or("unknown"),
        mapping
            .get("source")
            .and_then(Value::as_str)
            .unwrap_or("unknown"),
        mapping
            .get("destination")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
    )
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

    #[test]
    fn mapping_failure_has_stable_structured_details() {
        let error = GripError::mapping(
            "mapping_show",
            "mapping_not_found",
            vec!["/source".into()],
            "Mapping not found",
        );
        let outcome = CommandOutcome::failure(&error);
        assert_eq!(outcome.details["operation"], "mapping_show");
        assert_eq!(outcome.details["reason"], "mapping_not_found");
        assert_eq!(outcome.details["paths"][0], "/source");
    }

    #[test]
    fn publication_mapping_failures_use_the_operational_failure_envelope() {
        for reason in ["publication_failure", "registry_recovery_failure"] {
            let error = GripError::mapping(
                "mapping_update",
                reason,
                vec!["/grip/config.toml".into()],
                "Registry update failed",
            );
            let outcome = CommandOutcome::failure(&error);
            assert_eq!(outcome.category, ResultCategory::InternalError);
            assert_eq!(outcome.details["reason"], reason);
            let envelope = ResultEnvelopeV1::from(outcome);
            assert_eq!(envelope.code, "operational_failure");
            assert_eq!(envelope.status, "error");
            assert_eq!(envelope.details["operation"], "mapping_update");
        }
    }

    #[test]
    fn registry_io_mapping_failure_retains_operational_category_and_details() {
        let error = GripError::RegistryIo("could not read config.toml".into())
            .for_mapping_operation("mapping_list")
            .with_paths_if_empty(vec!["/grip/config.toml".into()]);
        let envelope = ResultEnvelopeV1::from(CommandOutcome::failure(&error));
        assert_eq!(envelope.code, "operational_failure");
        assert_eq!(envelope.details["operation"], "mapping_list");
        assert_eq!(envelope.details["reason"], "invalid_registry");
        assert_eq!(envelope.details["paths"][0], "/grip/config.toml");
    }

    #[test]
    fn unsupported_schema_mapping_failure_retains_category_and_details() {
        let error = GripError::UnsupportedSchema("unsupported registry schema version 2".into())
            .for_mapping_operation("mapping_show")
            .with_paths_if_empty(vec!["/grip/config.toml".into()]);
        let envelope = ResultEnvelopeV1::from(CommandOutcome::failure(&error));
        assert_eq!(envelope.code, "unsupported_schema");
        assert_eq!(envelope.details["operation"], "mapping_show");
        assert_eq!(envelope.details["reason"], "invalid_registry");
        assert_eq!(envelope.details["paths"][0], "/grip/config.toml");
    }
}
