use std::io;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultCategory {
    Success,
    InvalidUsage,
    InvalidConfiguration,
    UnsupportedSchema,
    CorruptState,
    InternalError,
}

impl ResultCategory {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Success => "ok",
            Self::InvalidUsage => "invalid_usage",
            Self::InvalidConfiguration => "invalid_configuration",
            Self::UnsupportedSchema => "unsupported_schema",
            Self::CorruptState => "corrupt_state",
            Self::InternalError => "operational_failure",
        }
    }
    pub const fn exit_code(self) -> u8 {
        match self {
            Self::Success => 0,
            Self::InvalidUsage => 2,
            Self::InvalidConfiguration => 10,
            Self::UnsupportedSchema => 11,
            Self::CorruptState => 12,
            Self::InternalError => 20,
        }
    }
    pub const fn status(self) -> &'static str {
        if matches!(self, Self::Success) {
            "ok"
        } else {
            "error"
        }
    }
}

#[derive(Debug, Error)]
pub enum GripError {
    #[error("{0}")]
    InvalidConfiguration(String),
    #[error("{0}")]
    UnsupportedSchema(String),
    #[error("{0}")]
    CorruptState(String),
    #[error("state publication is already in progress")]
    StateContention,
    #[error("{0}")]
    Internal(String),
}

impl GripError {
    pub fn category(&self) -> ResultCategory {
        match self {
            Self::InvalidConfiguration(_) => ResultCategory::InvalidConfiguration,
            Self::UnsupportedSchema(_) => ResultCategory::UnsupportedSchema,
            Self::CorruptState(_) => ResultCategory::CorruptState,
            Self::StateContention | Self::Internal(_) => ResultCategory::InternalError,
        }
    }
    pub fn from_io(context: &str, error: io::Error) -> Self {
        Self::Internal(format!("{context}: {error}"))
    }
}
