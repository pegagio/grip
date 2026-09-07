use crate::mapping::OwnershipConflict;
use std::io;
use thiserror::Error;

/// Structured terminal evidence for either mutation direction.
#[derive(Debug)]
pub struct MutationFailure {
    pub operation_id: String,
    pub plan: crate::mutation::model::MutationPlan,
    pub baseline: crate::mutation::model::BaselineOutcome,
    pub reason: String,
    pub phase: String,
    pub paths: Vec<crate::discovery::model::SafePath>,
    pub failed_action_index: Option<usize>,
    pub expected_source: Option<crate::observation::model::SupportedState>,
    pub expected_destination: Option<crate::observation::model::SupportedState>,
    pub observed_source: Option<crate::observation::model::SupportedState>,
    pub observed_destination: Option<crate::observation::model::SupportedState>,
    pub completion: String,
    pub category: ResultCategory,
    pub message: String,
}

/// Compatibility name retained for Feature 005 callers.
pub type PushFailure = MutationFailure;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultCategory {
    Success,
    AttentionRequired,
    InvalidUsage,
    InvalidConfiguration,
    UnsupportedSchema,
    CorruptState,
    StateContention,
    InternalError,
}

impl ResultCategory {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Success => "ok",
            Self::AttentionRequired => "attention_required",
            Self::InvalidUsage => "invalid_usage",
            Self::InvalidConfiguration => "invalid_configuration",
            Self::UnsupportedSchema => "unsupported_schema",
            Self::CorruptState => "corrupt_state",
            Self::StateContention => "state_contention",
            Self::InternalError => "operational_failure",
        }
    }
    pub const fn exit_code(self) -> u8 {
        match self {
            Self::Success => 0,
            Self::AttentionRequired => 1,
            Self::InvalidUsage => 2,
            Self::InvalidConfiguration => 10,
            Self::UnsupportedSchema => 11,
            Self::CorruptState => 12,
            Self::StateContention => 13,
            Self::InternalError => 20,
        }
    }
    pub const fn status(self) -> &'static str {
        if matches!(self, Self::Success | Self::AttentionRequired) {
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
    RegistryIo(String),
    #[error("{0}")]
    UnsupportedSchema(String),
    #[error("{0}")]
    CorruptState(String),
    #[error("state publication is already in progress")]
    StateContention,
    #[error("another Grip writer holds the mutation lock")]
    MutationContention {
        requested_operation: String,
        owner: Option<crate::state::mutation_lock::MutationLockOwner>,
    },
    #[error("selected baseline evidence is not complete and equivalent")]
    BaselineNotAcceptable {
        records: Vec<crate::classification::model::ClassificationRecord>,
    },
    #[error("{}", .0.message)]
    PushFailed(Box<PushFailure>),
    #[error("{}", .0.message)]
    MutationFailed(Box<MutationFailure>),
    #[error("{message}")]
    PushSideEffect {
        reason: String,
        publication_visible: bool,
        verification: String,
        durability_confirmed: bool,
        message: String,
    },
    #[error("{message}")]
    MutationSideEffect {
        reason: String,
        publication_visible: bool,
        verification: String,
        durability_confirmed: bool,
        message: String,
    },
    #[error("{message}")]
    Mapping {
        operation: String,
        reason: String,
        paths: Vec<String>,
        kind: Option<crate::mapping::MappingKind>,
        publication_visible: bool,
        category: ResultCategory,
        message: String,
        conflicts: Vec<OwnershipConflict>,
    },
    #[error("{message}")]
    Lifecycle {
        operation: String,
        reason: String,
        category: ResultCategory,
        publication_visible: bool,
        verification: String,
        durability_confirmed: bool,
        message: String,
    },
    #[error("{message}")]
    OperationLifecycle {
        operation: String,
        reason: String,
        category: ResultCategory,
        operation_id: String,
        details: Box<serde_json::Map<String, serde_json::Value>>,
        publication_visible: bool,
        verification: String,
        durability_confirmed: bool,
        message: Box<str>,
    },
    #[error("{0}")]
    Internal(String),
}

impl GripError {
    pub fn category(&self) -> ResultCategory {
        match self {
            Self::InvalidConfiguration(_) => ResultCategory::InvalidConfiguration,
            Self::RegistryIo(_) => ResultCategory::InternalError,
            Self::UnsupportedSchema(_) => ResultCategory::UnsupportedSchema,
            Self::CorruptState(_) => ResultCategory::CorruptState,
            Self::Mapping { category, .. } => *category,
            Self::Lifecycle { category, .. } | Self::OperationLifecycle { category, .. } => {
                *category
            }
            Self::StateContention | Self::MutationContention { .. } => {
                ResultCategory::StateContention
            }
            Self::BaselineNotAcceptable { .. } => ResultCategory::InvalidConfiguration,
            Self::PushFailed(failure) | Self::MutationFailed(failure) => failure.category,
            Self::PushSideEffect { .. } | Self::MutationSideEffect { .. } => {
                ResultCategory::InternalError
            }
            Self::Internal(_) => ResultCategory::InternalError,
        }
    }

    pub fn lifecycle(
        operation: &str,
        reason: &str,
        category: ResultCategory,
        message: impl Into<String>,
    ) -> Self {
        Self::Lifecycle {
            operation: operation.into(),
            reason: reason.into(),
            category,
            publication_visible: false,
            verification: "not_attempted".into(),
            durability_confirmed: false,
            message: message.into(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn operation_lifecycle(
        operation: &str,
        reason: &str,
        category: ResultCategory,
        operation_id: &str,
        details: serde_json::Map<String, serde_json::Value>,
        publication_visible: bool,
        verification: &str,
        durability_confirmed: bool,
        message: impl Into<String>,
    ) -> Self {
        Self::OperationLifecycle {
            operation: operation.into(),
            reason: reason.into(),
            category,
            operation_id: operation_id.into(),
            details: Box::new(details),
            publication_visible,
            verification: verification.into(),
            durability_confirmed,
            message: message.into().into_boxed_str(),
        }
    }

    pub fn push_side_effect(
        reason: &str,
        publication_visible: bool,
        verification: &str,
        durability_confirmed: bool,
        message: impl Into<String>,
    ) -> Self {
        Self::PushSideEffect {
            reason: reason.into(),
            publication_visible,
            verification: verification.into(),
            durability_confirmed,
            message: message.into(),
        }
    }

    pub fn mutation_side_effect(
        reason: &str,
        publication_visible: bool,
        verification: &str,
        durability_confirmed: bool,
        message: impl Into<String>,
    ) -> Self {
        Self::MutationSideEffect {
            reason: reason.into(),
            publication_visible,
            verification: verification.into(),
            durability_confirmed,
            message: message.into(),
        }
    }
    pub fn from_io(context: &str, error: io::Error) -> Self {
        Self::Internal(format!("{context}: {error}"))
    }

    pub fn mapping(operation: &str, reason: &str, paths: Vec<String>, message: &str) -> Self {
        let category = if matches!(
            reason,
            "registry_contention" | "publication_failure" | "registry_recovery_failure"
        ) {
            ResultCategory::InternalError
        } else {
            ResultCategory::InvalidConfiguration
        };
        Self::mapping_with_category(operation, reason, paths, category, message)
    }

    fn mapping_with_category(
        operation: &str,
        reason: &str,
        paths: Vec<String>,
        category: ResultCategory,
        message: &str,
    ) -> Self {
        Self::Mapping {
            operation: operation.into(),
            reason: reason.into(),
            paths,
            kind: None,
            publication_visible: false,
            category,
            message: message.into(),
            conflicts: Vec::new(),
        }
    }

    pub fn ownership_conflicts(conflicts: Vec<OwnershipConflict>) -> Self {
        Self::Mapping {
            operation: "registry_validate".into(),
            reason: "ownership_conflicts".into(),
            paths: Vec::new(),
            kind: None,
            publication_visible: false,
            category: ResultCategory::InvalidConfiguration,
            message: "Mapping ownership conflicts with the accepted registry".into(),
            conflicts,
        }
    }

    pub fn for_operation(self, value: &str) -> Self {
        match self {
            Self::RegistryIo(message) => {
                Self::mapping_operational(value, "invalid_registry", Vec::new(), &message)
            }
            mut error => {
                if let Self::Mapping { operation, .. } = &mut error {
                    *operation = value.into();
                }
                error
            }
        }
    }

    pub fn for_mapping_kind(mut self, value: crate::mapping::MappingKind) -> Self {
        if let Self::Mapping { kind, .. } = &mut self {
            *kind = Some(value);
        }
        self
    }

    pub fn for_mapping_operation(self, operation: &str) -> Self {
        match self {
            Self::InvalidConfiguration(message) => {
                Self::mapping(operation, "invalid_registry", Vec::new(), &message)
            }
            Self::RegistryIo(message) => {
                Self::mapping_operational(operation, "invalid_registry", Vec::new(), &message)
            }
            Self::UnsupportedSchema(message) => Self::mapping_with_category(
                operation,
                "invalid_registry",
                Vec::new(),
                ResultCategory::UnsupportedSchema,
                &message,
            ),
            other => other.for_operation(operation),
        }
    }

    fn mapping_operational(
        operation: &str,
        reason: &str,
        paths: Vec<String>,
        message: &str,
    ) -> Self {
        Self::mapping_with_category(
            operation,
            reason,
            paths,
            ResultCategory::InternalError,
            message,
        )
    }

    /// Construct a stable operational discovery failure.
    pub fn discovery_operational(
        operation: &str,
        reason: &str,
        paths: Vec<String>,
        message: &str,
    ) -> Self {
        Self::mapping_operational(operation, reason, paths, message)
    }

    /// Construct a stable invalid discovery-configuration failure.
    pub fn discovery_invalid(
        operation: &str,
        reason: &str,
        paths: Vec<String>,
        message: &str,
    ) -> Self {
        Self::mapping_with_category(
            operation,
            reason,
            paths,
            ResultCategory::InvalidConfiguration,
            message,
        )
    }

    pub fn with_publication_visible(mut self, value: bool) -> Self {
        if let Self::Mapping {
            publication_visible,
            ..
        } = &mut self
        {
            *publication_visible = value;
        }
        self
    }

    pub fn with_paths_if_empty(mut self, value: Vec<String>) -> Self {
        if let Self::Mapping { paths, .. } = &mut self
            && paths.is_empty()
        {
            *paths = value;
        }
        self
    }
}

#[cfg(test)]
mod discovery_tests {
    use super::*;

    #[test]
    fn discovery_policy_and_operational_failures_have_stable_categories() {
        for reason in ["invalid_policy", "non_utf8_policy"] {
            let error = GripError::discovery_invalid(
                "mapping_inspect",
                reason,
                vec!["/source/.gripignore".into()],
                "Invalid policy",
            );
            assert_eq!(error.category(), ResultCategory::InvalidConfiguration);
        }
        for reason in [
            "unreadable_policy",
            "directory_unreadable",
            "stale_discovery_evidence",
        ] {
            let error = GripError::discovery_operational(
                "mapping_inspect",
                reason,
                vec!["/source".into()],
                "Inspection failed",
            );
            assert_eq!(error.category(), ResultCategory::InternalError);
        }
    }
}
