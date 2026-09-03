#!/usr/bin/env python3
"""Provide deterministic mechanics shared by Diagram Roadmap review commands."""

from __future__ import annotations

import argparse
from collections import Counter
from datetime import datetime, timezone
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import subprocess
import sys
from types import ModuleType
from typing import Any


def _load_sibling() -> ModuleType:
    """Load the reviewed sibling loader without ambient import-path dependence."""
    sys.dont_write_bytecode = True
    path = Path(__file__).resolve().with_name("load_config.py")
    specification = importlib.util.spec_from_file_location("diagram_roadmap_load_config", path)
    if specification is None or specification.loader is None:
        raise RuntimeError("sibling loader could not be loaded")
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


load_config = _load_sibling()


SEVERITIES = ("Must-Address", "Recommendation", "Question")
CATEGORIES = {
    "brief": (
        "outcome-drift",
        "scope-drift",
        "constraint-conflict",
        "dependency-not-ready",
        "status-drift",
        "unresolved-evidence",
    ),
    "debrief": ("outcome-miss", "scope-creep", "constraint-violation", "roadmap-stale"),
    "sync": (
        "status-lagging",
        "orphan-spec",
        "phantom-entry",
        "dependency-contradiction",
        "superseded-ADR",
        "abandoned-but-active",
    ),
}
LIFECYCLE_STATUSES = (
    "undecided",
    "needs-info",
    "planned",
    "specced",
    "in-progress",
    "implemented",
    "verified",
    "deferred",
    "abandoned",
)
TIMESTAMP_PATTERN = re.compile(r"^\d{8}T\d{6}Z$")
ENTRY_PATTERN = re.compile(r"^###\s+(\d+)\s+[—-]\s+(.+?)(?:\s+\[status:\s*([^\]]+)\])?\s*$")
FIELD_PATTERN = re.compile(r"^-\s+\*\*([^*]+?)(?::)?\*\*:?\s*(.*)$")


ContractError = load_config.ContractError


def _compact(result: dict[str, Any]) -> None:
    """Write one compact UTF-8 JSON object."""
    sys.stdout.write(json.dumps(result, ensure_ascii=False, separators=(",", ":")) + "\n")


def _error(error: BaseException) -> None:
    """Write one bounded public diagnostic."""
    message = str(error) if isinstance(error, ContractError) else "unexpected internal failure"
    escaped = json.dumps(message, ensure_ascii=False)[1:-1]
    sys.stderr.write(f"Error: {escaped}\n")


def _git(project_root: Path, *arguments: str, check: bool = True) -> subprocess.CompletedProcess[bytes]:
    """Run Git without a shell and return byte-exact output."""
    result = subprocess.run(
        ["git", *arguments],
        cwd=project_root,
        capture_output=True,
        check=False,
    )
    if check and result.returncode != 0:
        diagnostic = result.stderr.decode("utf-8", errors="replace").strip()
        raise ContractError(diagnostic or f"git {' '.join(arguments)} failed")
    return result


def _relative_path(project_root: Path, value: str, label: str) -> tuple[Path, str]:
    """Validate and normalize a repository-contained path."""
    load_config._reject_lexically(value, label)
    target = load_config.require_contained(project_root / value, project_root, label)
    relative = target.relative_to(project_root).as_posix()
    if relative == ".":
        raise ContractError(f"{label} must not resolve to the project root")
    return target, relative.rstrip("/")


def _read_roadmap(project_root: Path, value: str) -> list[dict[str, Any]]:
    """Parse exact ledger fields needed for deterministic target candidates."""
    path, relative = _relative_path(project_root, value, "roadmap path")
    if not path.is_file() or path.is_symlink():
        raise ContractError("roadmap path must be an existing non-symlink regular file")
    entries: list[dict[str, Any]] = []
    current: dict[str, Any] | None = None
    for line in path.read_text(encoding="utf-8").splitlines():
        heading = ENTRY_PATTERN.match(line)
        if heading:
            if current is not None:
                entries.append(current)
            title = heading.group(2).strip()
            current = {
                "id": heading.group(1),
                "title": title,
                "status": (heading.group(3) or "").strip().lower() or None,
                "spec_dir": None,
                "spec_file": None,
            }
            continue
        if current is None:
            continue
        field = FIELD_PATTERN.match(line)
        if field is None:
            continue
        name = field.group(1).strip().lower().replace(" ", "-")
        value_text = field.group(2).strip().strip("`")
        if name == "status":
            current["status"] = value_text.lower()
        elif name == "spec-dir":
            normalized = value_text.rstrip("/")
            current["spec_dir"] = normalized
            current["spec_file"] = f"{normalized}/spec.md"
    if current is not None:
        entries.append(current)
    if not entries:
        raise ContractError(f"roadmap {relative} contains no ledger entries")
    return entries


def _normalize_spec_target(project_root: Path, value: str) -> str:
    """Normalize a spec directory or spec file to its directory."""
    _target, relative = _relative_path(project_root, value, "spec target")
    if relative.endswith("/spec.md"):
        relative = relative[: -len("/spec.md")]
    return relative


def resolve_target(project_root: Path, arguments: argparse.Namespace) -> dict[str, Any]:
    """Resolve exact targets or return candidates for command-owned judgment."""
    entries = _read_roadmap(project_root, arguments.roadmap_path)
    explicit = bool(arguments.spec_target or arguments.roadmap_entry)
    if not explicit and not arguments.ambient_feature:
        raise ContractError("an explicit target or ambient feature is required")
    spec_value = arguments.spec_target if explicit and arguments.spec_target else None
    entry_value = arguments.roadmap_entry if explicit and arguments.roadmap_entry else None
    if not explicit:
        spec_value = arguments.ambient_feature
    source = (
        "combined-explicit"
        if arguments.spec_target and arguments.roadmap_entry
        else "explicit-spec"
        if arguments.spec_target
        else "explicit-entry"
        if arguments.roadmap_entry
        else "ambient-feature"
    )
    spec_matches: list[dict[str, Any]] = []
    if spec_value:
        normalized = _normalize_spec_target(project_root, spec_value)
        spec_matches = [entry for entry in entries if entry["spec_dir"] == normalized]
    entry_matches: list[dict[str, Any]] = []
    match_method = "exact-spec-dir"
    if entry_value:
        exact_id = [entry for entry in entries if entry["id"] == entry_value]
        exact_title = [entry for entry in entries if entry["title"].casefold() == entry_value.casefold()]
        entry_matches = exact_id or exact_title
        match_method = "number-only" if exact_id else "title-similarity"
    if spec_value and entry_value:
        intersection = [entry for entry in spec_matches if entry in entry_matches]
        if not intersection:
            return {"state": "conflict", "selection_source": source, "candidates": [], "conflicts": ["explicit spec target and roadmap entry do not converge"]}
        selected = intersection[0]
        match_method = "exact-spec-dir"
    elif spec_value and spec_matches:
        selected = spec_matches[0]
        match_method = "exact-spec-dir"
    elif entry_value and entry_matches:
        selected = entry_matches[0]
    else:
        selected = None
    if selected is not None:
        return {"state": "selected", "selection_source": source, "match_method": match_method, "entry": selected, "conflicts": []}
    candidates = [entry for entry in entries if entry["spec_dir"]]
    if arguments.selected_entry:
        chosen = next((entry for entry in candidates if entry["id"] == arguments.selected_entry), None)
        if chosen is None:
            raise ContractError("selected entry is not an eligible target candidate")
        return {"state": "selected", "selection_source": source, "match_method": "title-similarity", "entry": chosen, "conflicts": []}
    return {"state": "needs-judgment", "selection_source": source, "candidates": candidates, "conflicts": []}


def _resolve_commit(project_root: Path, reference: str, label: str) -> str:
    """Resolve one revision to an immutable commit identifier."""
    if not reference or reference.startswith("-"):
        raise ContractError(f"{label} must be a non-option Git reference")
    result = _git(project_root, "rev-parse", "--verify", f"{reference}^{{commit}}")
    return result.stdout.decode().strip()


def _status_name(code: str) -> str:
    """Map Git name-status codes to stable names."""
    return {
        "A": "added",
        "M": "modified",
        "D": "deleted",
        "R": "renamed",
        "C": "copied",
        "T": "type-changed",
        "U": "unmerged",
    }.get(code[:1], "unknown")


def _add_artifact(store: dict[tuple[str | None, str], dict[str, Any]], status: str, old_path: str | None, new_path: str, source: str) -> None:
    """Merge one artifact observation without losing its sources."""
    key = (old_path, new_path)
    artifact = store.setdefault(key, {"status": status, "old_path": old_path, "new_path": new_path, "sources": []})
    if source not in artifact["sources"]:
        artifact["sources"].append(source)


def _parse_name_status(raw: bytes, source: str, store: dict[tuple[str | None, str], dict[str, Any]]) -> None:
    """Parse `git diff --name-status -z` output."""
    parts = raw.decode("utf-8", errors="surrogateescape").split("\0")
    index = 0
    while index < len(parts) and parts[index]:
        code = parts[index]
        index += 1
        status = _status_name(code)
        if status in {"renamed", "copied"}:
            if index + 1 >= len(parts):
                raise ContractError("git returned an incomplete rename record")
            old_path, new_path = parts[index], parts[index + 1]
            index += 2
        else:
            if index >= len(parts):
                raise ContractError("git returned an incomplete path record")
            old_path, new_path = None, parts[index]
            index += 1
        _add_artifact(store, status, old_path, new_path, source)


def _dirty_snapshot(project_root: Path) -> tuple[bytes, dict[str, int]]:
    """Capture porcelain state and deterministic category counts."""
    raw = _git(project_root, "status", "--porcelain=v1", "-z", "--untracked-files=all").stdout
    counts = {"staged": 0, "unstaged": 0, "untracked": 0, "deleted": 0, "renamed": 0, "gitlinks": 0}
    records = [record for record in raw.decode("utf-8", errors="surrogateescape").split("\0") if record]
    for record in records:
        code = record[:2]
        if code == "??":
            counts["untracked"] += 1
            continue
        if code[0] != " ":
            counts["staged"] += 1
        if code[1] != " ":
            counts["unstaged"] += 1
        if "D" in code:
            counts["deleted"] += 1
        if "R" in code:
            counts["renamed"] += 1
    return raw, counts


def _mark_gitlinks(project_root: Path, artifacts: list[dict[str, Any]], dirty_counts: dict[str, int]) -> None:
    """Classify changed index entries whose Git mode is a gitlink."""
    changed = 0
    for artifact in artifacts:
        result = _git(project_root, "ls-files", "-s", "--", artifact["new_path"], check=False)
        if result.returncode == 0 and any(line.startswith(b"160000 ") for line in result.stdout.splitlines()):
            artifact["status"] = "gitlink"
            changed += 1
    dirty_counts["gitlinks"] = changed


def _snapshot_bytes(project_root: Path, head: str, dirty_raw: bytes, artifacts: list[dict[str, Any]]) -> bytes:
    """Capture content-sensitive revision, index, worktree, and untracked identity."""
    material = bytearray(head.encode("ascii") + b"\0" + dirty_raw)
    material.extend(_git(project_root, "diff", "--binary", "--cached", "--").stdout)
    material.extend(_git(project_root, "diff", "--binary", "--").stdout)
    for artifact in artifacts:
        if "untracked" not in artifact["sources"]:
            continue
        path = load_config.require_contained(project_root / artifact["new_path"], project_root, "untracked snapshot path")
        material.extend(artifact["new_path"].encode("utf-8", errors="surrogateescape") + b"\0")
        if path.is_file() and not path.is_symlink():
            material.extend(hashlib.sha256(path.read_bytes()).digest())
    return bytes(material)


def resolve_delta(project_root: Path, arguments: argparse.Namespace) -> dict[str, Any]:
    """Resolve a commit or working-tree implementation delta."""
    head = _resolve_commit(project_root, "HEAD", "HEAD")
    dirty_raw, dirty_counts = _dirty_snapshot(project_root)
    dirty = bool(dirty_raw)
    baseline_input = arguments.baseline
    target_input = arguments.target
    if bool(baseline_input) != bool(target_input):
        raise ContractError("baseline and target must be supplied together")
    artifacts: dict[tuple[str | None, str], dict[str, Any]] = {}
    exclusions: list[str] = []
    limitations: list[str] = []
    baseline_oid: str | None = None
    target_oid: str | None = None
    ancestry: bool | None = None
    if baseline_input and target_input:
        baseline_oid = _resolve_commit(project_root, baseline_input, "baseline")
        if target_input == "WORKTREE":
            mode = "baseline-to-worktree"
            ancestry = True
            _parse_name_status(_git(project_root, "diff", "--name-status", "-z", "-M", f"{baseline_oid}..{head}", "--").stdout, "commit-range", artifacts)
        else:
            mode = "explicit-commit-range"
            target_oid = _resolve_commit(project_root, target_input, "target")
            ancestry_result = _git(project_root, "merge-base", "--is-ancestor", baseline_oid, target_oid, check=False)
            ancestry = ancestry_result.returncode == 0
            if not ancestry:
                raise ContractError("baseline must be an ancestor of target")
            _parse_name_status(_git(project_root, "diff", "--name-status", "-z", "-M", f"{baseline_oid}..{target_oid}", "--").stdout, "commit-range", artifacts)
            if dirty:
                exclusions.append("ambient working-tree changes were excluded from the explicit commit range")
    elif dirty:
        mode = "head-to-worktree"
        baseline_input = "HEAD"
        baseline_oid = head
        target_input = "WORKTREE"
        ancestry = True
    else:
        mode = "unavailable"
        limitations.append("clean repository and no explicit implementation range")
    if mode in {"baseline-to-worktree", "head-to-worktree"}:
        _parse_name_status(_git(project_root, "diff", "--cached", "--name-status", "-z", "-M", "--").stdout, "index", artifacts)
        _parse_name_status(_git(project_root, "diff", "--name-status", "-z", "-M", "--").stdout, "worktree", artifacts)
        untracked = _git(project_root, "ls-files", "--others", "--exclude-standard", "-z").stdout.decode("utf-8", errors="surrogateescape")
        for path in sorted(item for item in untracked.split("\0") if item):
            _add_artifact(artifacts, "added", None, path, "untracked")
    normalized = sorted(artifacts.values(), key=lambda item: (item["new_path"], item["old_path"] or "", item["status"]))
    _mark_gitlinks(project_root, normalized, dirty_counts)
    digest_input = _snapshot_bytes(project_root, head, dirty_raw, normalized)
    return {
        "mode": mode,
        "baseline_input": baseline_input,
        "baseline_oid": baseline_oid,
        "target_input": target_input,
        "target_oid": target_oid,
        "head_oid": head,
        "ancestry_valid": ancestry,
        "dirty_state": dirty_counts,
        "artifacts": normalized,
        "exclusions": exclusions,
        "limitations": limitations,
        "snapshot_digest": f"sha256:{hashlib.sha256(digest_input).hexdigest()}",
        "trustworthy": mode != "unavailable",
    }


def _count_map(values: list[str], displayed_values: list[str], allowed: tuple[str, ...]) -> dict[str, dict[str, int]]:
    """Build complete, displayed, and omitted counts in canonical order."""
    totals = Counter(values)
    displayed = Counter(displayed_values)
    keys = [key for key in allowed if totals[key]]
    return {key: {"total": totals[key], "displayed": displayed[key], "omitted": totals[key] - displayed[key]} for key in keys}


def evaluate_findings(arguments: argparse.Namespace) -> dict[str, Any]:
    """Validate, order, cap, count, and derive a verdict from findings."""
    try:
        findings = json.load(sys.stdin)
    except (json.JSONDecodeError, UnicodeDecodeError) as error:
        raise ContractError("findings input must be one valid JSON array") from error
    if type(findings) is not list:
        raise ContractError("findings input must be a JSON array")
    validated: list[dict[str, Any]] = []
    for index, finding in enumerate(findings):
        if type(finding) is not dict:
            raise ContractError("each finding must be an object")
        severity = finding.get("severity")
        category = finding.get("category")
        if severity not in SEVERITIES:
            raise ContractError("finding severity is not canonical")
        if category not in CATEGORIES[arguments.kind]:
            raise ContractError("finding category is not valid for the review kind")
        if type(finding.get("text")) is not str or not finding["text"].strip():
            raise ContractError("finding text must be non-empty")
        if type(finding.get("suggestion")) is not str or not finding["suggestion"].strip():
            raise ContractError("finding suggestion must be non-empty")
        evidence = finding.get("evidence")
        if type(evidence) is not list or not evidence or any(type(item) is not str or not item for item in evidence):
            raise ContractError("finding evidence must be a non-empty string array")
        blocking = finding.get("blocking")
        if type(blocking) is not bool:
            raise ContractError("finding blocking must be boolean")
        if blocking and severity != "Must-Address":
            raise ContractError("only Must-Address findings may be blocking")
        validated.append({**finding, "input_order": index})
    validated.sort(key=lambda item: (SEVERITIES.index(item["severity"]), item["input_order"]))
    for index, finding in enumerate(validated, 1):
        finding["id"] = f"F{index}"
    displayed = validated[: arguments.max_findings]
    verdict = "RETHINK" if any(item["severity"] == "Must-Address" for item in validated) else "PROCEED WITH UPDATES" if validated else "PROCEED"
    return {
        "findings": displayed,
        "summary": {
            "max_findings": arguments.max_findings,
            "total": len(validated),
            "displayed": len(displayed),
            "omitted": len(validated) - len(displayed),
            "by_severity": _count_map([item["severity"] for item in validated], [item["severity"] for item in displayed], SEVERITIES),
            "by_category": _count_map([item["category"] for item in validated], [item["category"] for item in displayed], CATEGORIES[arguments.kind]),
            "verdict": verdict,
        },
    }


def _boolean(value: str | None, label: str, *, required: bool = False) -> bool | None:
    """Parse one strict command-line boolean."""
    if value is None and not required:
        return None
    if value not in {"true", "false"}:
        raise ContractError(f"{label} must be true or false")
    return value == "true"


def evaluate_lifecycle(arguments: argparse.Namespace) -> dict[str, Any]:
    """Evaluate deterministic brief or debrief lifecycle gates."""
    status = arguments.status
    if status not in LIFECYCLE_STATUSES:
        raise ContractError("status is not in the roadmap lifecycle")
    if arguments.phase == "brief":
        if arguments.outcome_met is not None or arguments.delta_trustworthy is not None or arguments.must_address_count is not None:
            raise ContractError("debrief-only lifecycle inputs are invalid for brief")
        complete = _boolean(arguments.spec_complete, "spec-complete")
        dependencies = arguments.dependency_status or []
        if any(item not in LIFECYCLE_STATUSES for item in dependencies):
            raise ContractError("dependency status is not in the roadmap lifecycle")
        ready = all(item == "verified" for item in dependencies)
        awaiting = [item for item in dependencies if item == "implemented"]
        blocked = [item for item in dependencies if item != "verified"]
        proposed: str | None = None
        approval = False
        if status == "planned" and complete is True:
            proposed = "specced"
            reason = "The planned entry has a complete specification."
        elif status == "specced" and ready:
            proposed = "in-progress"
            reason = "All dependencies are verified."
        elif status in {"deferred", "abandoned"}:
            approval = True
            reason = "Explicit reactivation approval is required."
        elif status in {"undecided", "needs-info"}:
            reason = "Missing information must be resolved before transition."
        elif status in {"in-progress", "implemented", "verified"}:
            reason = "No redundant lifecycle transition is needed."
        elif status == "planned":
            reason = "A complete specification is required before specced."
        else:
            reason = "Dependencies are not ready."
        return {"current_status": status, "proposed_status": proposed, "gate_results": {"spec_complete": complete, "dependencies_ready": ready, "awaiting_verification": awaiting, "blocked_dependencies": blocked}, "approval_required": approval, "reason": reason}
    if arguments.spec_complete is not None or arguments.dependency_status:
        raise ContractError("brief-only lifecycle inputs are invalid for debrief")
    outcome = _boolean(arguments.outcome_met, "outcome-met", required=True)
    trustworthy = _boolean(arguments.delta_trustworthy, "delta-trustworthy", required=True)
    if arguments.must_address_count is None or arguments.must_address_count < 0:
        raise ContractError("must-address-count must be a non-negative integer")
    eligible = status in {"in-progress", "implemented"}
    clear = arguments.must_address_count == 0
    proposed = "verified" if eligible and outcome and trustworthy and clear else None
    return {"current_status": status, "proposed_status": proposed, "gate_results": {"eligible_source_status": eligible, "outcome_met": outcome, "delta_trustworthy": trustworthy, "must_address_clear": clear}, "approval_required": False, "reason": "All verification recommendation gates passed." if proposed else "One or more verification recommendation gates failed."}


def allocate_report(project_root: Path, arguments: argparse.Namespace) -> dict[str, Any]:
    """Atomically reserve a unique report path within its permitted directory."""
    timestamp = arguments.timestamp_utc or datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    if not TIMESTAMP_PATTERN.fullmatch(timestamp):
        raise ContractError("timestamp-utc must use YYYYMMDDTHHMMSSZ")
    if arguments.kind == "sync":
        if arguments.feature_dir:
            raise ContractError("sync does not accept feature-dir")
        directory = project_root / ".specify" / "memory" / "roadmap-reviews"
    else:
        if not arguments.feature_dir:
            raise ContractError("brief and debrief require feature-dir")
        feature, _relative = _relative_path(project_root, arguments.feature_dir, "feature directory")
        directory = feature / "roadmap-reviews"
    directory = load_config.require_contained(directory, project_root, "report directory")
    directory.mkdir(parents=True, exist_ok=True)
    for collision in range(1, 10_000):
        suffix = "" if collision == 1 else f"-{collision}"
        candidate = load_config.require_contained(directory / f"{arguments.kind}-{timestamp}{suffix}.md", project_root, "report path")
        try:
            descriptor = os.open(candidate, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
            os.close(descriptor)
        except FileExistsError:
            continue
        return {"state": "reserved", "kind": arguments.kind, "timestamp_utc": timestamp, "collision_index": collision, "path": candidate.relative_to(project_root).as_posix()}
    raise ContractError("report collision limit was exceeded")


def _parser() -> argparse.ArgumentParser:
    """Build the deterministic command-line grammar."""
    parser = argparse.ArgumentParser(add_help=False)
    subparsers = parser.add_subparsers(dest="operation")
    path = subparsers.add_parser("validate-path", add_help=False)
    path.add_argument("--kind", choices=tuple(sorted(load_config.VALIDATION_KINDS)), required=True)
    path.add_argument("--path", required=True)
    target = subparsers.add_parser("resolve-target", add_help=False)
    target.add_argument("--command", choices=("brief", "debrief"), required=True)
    target.add_argument("--roadmap-path", required=True)
    target.add_argument("--spec-target")
    target.add_argument("--roadmap-entry")
    target.add_argument("--ambient-feature")
    target.add_argument("--selected-entry")
    delta = subparsers.add_parser("resolve-delta", add_help=False)
    delta.add_argument("--baseline")
    delta.add_argument("--target")
    findings = subparsers.add_parser("evaluate-findings", add_help=False)
    findings.add_argument("--kind", choices=tuple(CATEGORIES), required=True)
    findings.add_argument("--max-findings", type=int, required=True)
    lifecycle = subparsers.add_parser("evaluate-lifecycle", add_help=False)
    lifecycle.add_argument("--phase", choices=("brief", "debrief"), required=True)
    lifecycle.add_argument("--status", required=True)
    lifecycle.add_argument("--spec-complete")
    lifecycle.add_argument("--dependency-status", action="append")
    lifecycle.add_argument("--outcome-met")
    lifecycle.add_argument("--delta-trustworthy")
    lifecycle.add_argument("--must-address-count", type=int)
    report = subparsers.add_parser("allocate-report", add_help=False)
    report.add_argument("--kind", choices=tuple(CATEGORIES), required=True)
    report.add_argument("--feature-dir")
    report.add_argument("--timestamp-utc")
    return parser


def _run(yaml: Any) -> dict[str, Any]:
    """Dispatch one review-contract operation."""
    project_root, payload_root = load_config.discover_roots(Path(__file__))
    if not sys.argv[1:]:
        return load_config.resolve_configuration(project_root, payload_root, yaml)
    try:
        arguments = _parser().parse_args()
    except SystemExit as error:
        raise ContractError("invalid review_contract.py arguments") from error
    if arguments.operation == "resolve-target":
        return resolve_target(project_root, arguments)
    if arguments.operation == "validate-path":
        return load_config.validate_concrete_path(project_root, arguments.kind, arguments.path)
    if arguments.operation == "resolve-delta":
        return resolve_delta(project_root, arguments)
    if arguments.operation == "evaluate-findings":
        if arguments.max_findings < 0:
            raise ContractError("max-findings must be a non-negative integer")
        return evaluate_findings(arguments)
    if arguments.operation == "evaluate-lifecycle":
        return evaluate_lifecycle(arguments)
    if arguments.operation == "allocate-report":
        return allocate_report(project_root, arguments)
    raise ContractError("a review contract operation is required")


def main() -> int:
    """Run behind the stable public success and failure boundary."""
    try:
        yaml = load_config.establish_runtime(Path(__file__))
        _compact(_run(yaml))
        return 0
    except BaseException as error:
        if isinstance(error, (KeyboardInterrupt, SystemExit)):
            raise
        _error(error)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
