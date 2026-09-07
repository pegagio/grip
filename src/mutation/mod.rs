//! Direction-neutral mutation planning and execution primitives.

pub mod execution;
pub mod filesystem;
pub mod model;
pub mod plan;
pub mod recovery;

/// Test-only fault boundaries in the mutating pipeline.
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
    BeforeTargetVerification(usize),
    AfterTargetVerification(usize),
    AfterStagingCleanup(usize),
    /// Compatibility boundary retained for Feature 005 push fault tests.
    BeforeDestinationVerification(usize),
    BeforeFinalObservation,
    BeforeFinalCoordination,
    BeforeBaselinePublication,
    AfterBaselinePublication,
    BeforeTerminalSummary,
}
