# Implementation Plan: Global Ignore Policy and Directory Timestamp Boundary

## Technical Context

Grip is a Rust CLI with non-following tree discovery, Gitignore-compatible policy matching, complete metadata observations, three-way classification, and staged mutation planning.

## Design

Discovery loads the project-root `.gripignore` once per tree inspection and supplies it as the inherited lowest-precedence policy. Existing source-root and nested policy loading continues to append policies, so reverse precedence evaluation selects the narrowest matching policy.

Complete directory observations retain their timestamp as diagnostic evidence for backward-compatible state decoding, but all managed equality, changed-dimension reporting, mutation capability checks, metadata transfer, and verification compare directory state with timestamp ignored. File behavior remains unchanged.

## Constitution Check

The change flows forward from Feature 003's source discovery and Feature 009's metadata contract. It preserves project-scoped operation, source-defined ownership, no-follow inspection, and bounded mutation verification.

## Validation

Run focused policy and classification tests, then the repository validation task. Confirm a global ignore rule is inherited by a descendant tree mapping and that a directory-only timestamp difference is synchronized.
