//! Safe source-to-destination push planning and execution.

pub mod execution;
pub mod filesystem;
pub mod model;
pub mod plan;
pub mod recovery;

/// Test-only fault boundaries in the mutating push pipeline.
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaultPhase {
    AfterMutationLock,
    BeforeActionRevalidation(usize),
    BeforeRecovery(usize),
    AfterRecovery(usize),
    BeforeStaging(usize),
    AfterStaging(usize),
    BeforePayloadPublication(usize),
    AfterPayloadPublication(usize),
    BeforeDestinationVerification(usize),
    BeforeFinalObservation,
    BeforeFinalCoordination,
    BeforeBaselinePublication,
    AfterBaselinePublication,
    BeforeTerminalSummary,
}
