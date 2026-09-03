#!/usr/bin/env python3
"""Load and validate Diagram Roadmap configuration as a stable JSON contract."""

from __future__ import annotations

import csv
import io
import json
import os
from pathlib import Path
import re
import shutil
import stat
import sys
from typing import Any


REQUIRED_PYTHON = (3, 11, 16)
REQUIRED_PYYAML = (6, 0)
MAXIMUM_YAML_BYTES = 65_536
REEXECUTION_MARKER = "SPECKIT_DIAGRAM_ROADMAP_ISOLATED"
VALIDATION_KINDS = {"roadmap-read", "roadmap-write", "adr", "prd"}
DEFAULT_ROADMAP_PATH = ".specify/memory/roadmap.md"
DEFAULT_ADR_DIRECTORY = "docs/adr/"
DEFAULT_PRD_GLOBS = [
    "**/prd*.md",
    "**/PRD*.md",
    "**/prd-intake.yaml",
    "**/product-spec.md",
    "docs/product/**/*.md",
]
DEFAULT_MAX_FINDINGS = 50


class ContractError(Exception):
    """A bounded failure that is safe to expose through the public error contract."""


def _version_tuple(raw_version: str) -> tuple[int, ...]:
    """Parse the leading numeric components from a package version."""
    match = re.match(r"^(\d+)(?:\.(\d+))?(?:\.(\d+))?", raw_version)
    if match is None:
        raise ContractError("PyYAML version could not be verified")
    return tuple(int(part or 0) for part in match.groups())


def _resolve_specify_interpreter() -> tuple[Path, Path, Path | None]:
    """Resolve the invocation path and identity of the active Specify interpreter."""
    selected = shutil.which("specify")
    if selected is None:
        raise ContractError("active specify executable was not found on PATH")
    try:
        executable = Path(selected).resolve(strict=True)
        executable_stat = executable.stat()
    except (OSError, RuntimeError) as error:
        raise ContractError("active specify executable could not be resolved") from error
    if not stat.S_ISREG(executable_stat.st_mode) or not os.access(executable, os.X_OK):
        raise ContractError("active specify executable is not an executable regular file")
    try:
        with executable.open("rb") as stream:
            first_line = stream.readline(4097)
    except OSError as error:
        raise ContractError("active specify console-script entrypoint could not be read") from error
    try:
        shebang = first_line.decode("utf-8").rstrip("\r\n")
    except UnicodeDecodeError as error:
        raise ContractError("active specify executable is not a supported console-script entrypoint") from error
    if not shebang.startswith("#!"):
        raise ContractError("active specify executable is not a supported console-script entrypoint")
    interpreter_text = shebang[2:]
    if not interpreter_text or any(character.isspace() for character in interpreter_text):
        raise ContractError("active specify executable is not a supported console-script entrypoint")
    invocation_path = Path(interpreter_text)
    if not invocation_path.is_absolute():
        raise ContractError("active specify console-script interpreter is not absolute")
    try:
        canonical_path = invocation_path.resolve(strict=True)
        interpreter_stat = canonical_path.stat()
    except (OSError, RuntimeError) as error:
        raise ContractError("active specify interpreter could not be resolved") from error
    if not stat.S_ISREG(interpreter_stat.st_mode) or not os.access(invocation_path, os.X_OK):
        raise ContractError("active specify interpreter is not an executable regular file")
    prefix_candidate = invocation_path.parent.parent
    expected_prefix = prefix_candidate if (prefix_candidate / "pyvenv.cfg").is_file() else None
    return invocation_path, canonical_path, expected_prefix


def establish_runtime(script_path: Path | None = None) -> Any:
    """Transfer once to the isolated active Specify interpreter and import PyYAML."""
    invocation_path, canonical_path, expected_prefix = _resolve_specify_interpreter()
    try:
        current_interpreter = Path(sys.executable).resolve(strict=True)
    except (OSError, RuntimeError) as error:
        raise ContractError("current Python interpreter could not be resolved") from error
    prefix_matches = expected_prefix is None or Path(sys.prefix).resolve() == expected_prefix.resolve()
    requires_transfer = (
        current_interpreter != canonical_path or not prefix_matches or not sys.flags.isolated
    )
    if requires_transfer:
        if os.environ.get(REEXECUTION_MARKER) == "1":
            if sys.version_info[:3] != REQUIRED_PYTHON:
                actual = ".".join(str(part) for part in sys.version_info[:3])
                raise ContractError(f"active Specify Python must be 3.11.16, found {actual}")
            raise ContractError("isolated Specify interpreter transfer did not converge")
        environment = os.environ.copy()
        environment[REEXECUTION_MARKER] = "1"
        arguments = [
            str(invocation_path),
            "-I",
            str((script_path or Path(__file__)).resolve()),
            *sys.argv[1:],
        ]
        try:
            os.execve(str(invocation_path), arguments, environment)
        except OSError as error:
            raise ContractError("active Specify interpreter could not execute the loader") from error
    if sys.version_info[:3] != REQUIRED_PYTHON:
        actual = ".".join(str(part) for part in sys.version_info[:3])
        raise ContractError(f"active Specify Python must be 3.11.16, found {actual}")
    try:
        import yaml
    except (ImportError, ModuleNotFoundError) as error:
        raise ContractError("active Specify runtime does not provide PyYAML") from error
    if _version_tuple(getattr(yaml, "__version__", "")) < REQUIRED_PYYAML:
        raise ContractError("active Specify runtime requires PyYAML 6.0 or newer")
    return yaml


def _establish_runtime() -> Any:
    """Preserve the original private runtime entrypoint for compatibility."""
    return establish_runtime()


def _discover_roots() -> tuple[Path, Path]:
    """Derive project and payload roots from one of the two accepted layouts."""
    try:
        script = Path(__file__).resolve(strict=True)
    except (OSError, RuntimeError) as error:
        raise ContractError("loader script location could not be resolved") from error
    if script.parent.name != "python" or script.parent.parent.name != "scripts":
        raise ContractError("loader script is outside a supported source or installed layout")
    payload_candidate = script.parents[2]
    if (
        payload_candidate.name == "diagram-roadmap"
        and payload_candidate.parent.name == "extensions"
        and payload_candidate.parent.parent.name == ".specify"
    ):
        project_root = payload_candidate.parents[2]
        payload_root = payload_candidate
    else:
        project_root = payload_candidate
        payload_root = payload_candidate
    try:
        project_root = project_root.resolve(strict=True)
        payload_root = payload_root.resolve(strict=True)
    except (OSError, RuntimeError) as error:
        raise ContractError("project or payload root could not be resolved") from error
    return project_root, payload_root


def discover_roots(script_path: Path | None = None) -> tuple[Path, Path]:
    """Derive project and payload roots for a supported source or installed script."""
    if script_path is None or script_path.resolve() == Path(__file__).resolve():
        return _discover_roots()
    script = script_path.resolve(strict=True)
    if script.parent.name != "python" or script.parent.parent.name != "scripts":
        raise ContractError("script is outside a supported source or installed layout")
    payload_candidate = script.parents[2]
    if payload_candidate.name == "diagram-roadmap" and payload_candidate.parent.name == "extensions" and payload_candidate.parent.parent.name == ".specify":
        return payload_candidate.parents[2].resolve(strict=True), payload_candidate.resolve(strict=True)
    return payload_candidate.resolve(strict=True), payload_candidate.resolve(strict=True)


def _require_contained(candidate: Path, root: Path, label: str) -> Path:
    """Resolve a path and require it to remain at or below the supplied root."""
    try:
        resolved = candidate.resolve(strict=False)
        resolved.relative_to(root)
    except (OSError, RuntimeError, ValueError) as error:
        raise ContractError(f"{label} resolves outside the active project") from error
    return resolved


def require_contained(candidate: Path, root: Path, label: str) -> Path:
    """Expose the loader's canonical containment boundary to sibling scripts."""
    return _require_contained(candidate, root, label)


def _read_fixed_yaml(path: Path, root: Path, label: str, *, required: bool) -> str | None:
    """Read a bounded fixed YAML input without following its final symlink."""
    canonical = _require_contained(path, root, label)
    try:
        path_stat = path.lstat()
    except FileNotFoundError:
        if required:
            raise ContractError(f"{label} is missing")
        return None
    except OSError as error:
        raise ContractError(f"{label} could not be inspected") from error
    if stat.S_ISLNK(path_stat.st_mode) or not stat.S_ISREG(path_stat.st_mode):
        raise ContractError(f"{label} must be a non-symlink regular file")
    flags = os.O_RDONLY
    if hasattr(os, "O_NOFOLLOW"):
        flags |= os.O_NOFOLLOW
    try:
        descriptor = os.open(path, flags)
        try:
            opened_stat = os.fstat(descriptor)
            if not stat.S_ISREG(opened_stat.st_mode):
                raise ContractError(f"{label} must be a regular file")
            content = os.read(descriptor, MAXIMUM_YAML_BYTES + 1)
        finally:
            os.close(descriptor)
    except ContractError:
        raise
    except OSError as error:
        raise ContractError(f"{label} could not be read safely") from error
    if len(content) > MAXIMUM_YAML_BYTES:
        raise ContractError(f"{label} exceeds {MAXIMUM_YAML_BYTES} bytes")
    try:
        return content.decode("utf-8")
    except UnicodeDecodeError as error:
        raise ContractError(f"{label} is not valid UTF-8") from error


def _load_yaml(text: str | None, label: str, yaml: Any) -> Any:
    """Parse one YAML document while rejecting duplicate keys, aliases, and merges."""
    if text is None:
        return None

    class StrictSafeLoader(yaml.SafeLoader):
        def compose_node(self, parent: Any, index: Any) -> Any:
            if self.check_event(yaml.events.AliasEvent):
                raise ContractError(f"{label} must not contain YAML aliases")
            return super().compose_node(parent, index)

        def construct_mapping(self, node: Any, deep: bool = False) -> dict[Any, Any]:
            if not isinstance(node, yaml.nodes.MappingNode):
                raise ContractError(f"{label} contains an invalid mapping")
            mapping: dict[Any, Any] = {}
            for key_node, value_node in node.value:
                if key_node.tag == "tag:yaml.org,2002:merge" or key_node.value == "<<":
                    raise ContractError(f"{label} must not contain YAML merge keys")
                key = self.construct_object(key_node, deep=deep)
                try:
                    duplicate = key in mapping
                except TypeError as error:
                    raise ContractError(f"{label} contains an invalid mapping key") from error
                if duplicate:
                    raise ContractError(f"{label} contains duplicate key {key!r}")
                mapping[key] = self.construct_object(value_node, deep=deep)
            return mapping

    StrictSafeLoader.add_constructor(
        yaml.resolver.BaseResolver.DEFAULT_MAPPING_TAG,
        StrictSafeLoader.construct_mapping,
    )
    try:
        documents = list(yaml.load_all(text, Loader=StrictSafeLoader))
    except ContractError:
        raise
    except yaml.YAMLError as error:
        mark = getattr(error, "problem_mark", None)
        location = f" at line {mark.line + 1}, column {mark.column + 1}" if mark else ""
        raise ContractError(f"{label} contains invalid YAML{location}") from error
    if not documents:
        return None
    if len(documents) != 1:
        raise ContractError(f"{label} must contain exactly one YAML document")
    return documents[0]


def _validate_document(document: Any, label: str) -> dict[str, Any]:
    """Validate the exact four-section configuration structure."""
    if document is None:
        return {}
    if type(document) is not dict:
        raise ContractError(f"{label} root must be a mapping")
    allowed = {
        "roadmap": {"path"},
        "adr": {"dir"},
        "prd": {"globs"},
        "report": {"max_findings"},
    }
    unknown_sections = set(document) - set(allowed)
    if unknown_sections:
        raise ContractError(f"{label} contains unknown section {sorted(unknown_sections)[0]!r}")
    result: dict[str, Any] = {}
    for section, leaves in allowed.items():
        value = document.get(section)
        if value is None:
            continue
        if type(value) is not dict:
            raise ContractError(f"{label} section {section!r} must be a mapping or null")
        unknown_leaves = set(value) - leaves
        if unknown_leaves:
            raise ContractError(f"{label} contains unknown key {section}.{sorted(unknown_leaves)[0]}")
        result[section] = dict(value)
    for section, leaf in (("roadmap", "path"), ("adr", "dir")):
        value = result.get(section, {}).get(leaf)
        if value is not None and type(value) is not str:
            raise ContractError(f"{label} key {section}.{leaf} must be a string or null")
    globs = result.get("prd", {}).get("globs")
    if globs is not None:
        if type(globs) is not list:
            raise ContractError(f"{label} key prd.globs must be a list or null")
        if any(type(item) is not str or not item for item in globs):
            raise ContractError(f"{label} key prd.globs must contain non-empty strings")
    maximum = result.get("report", {}).get("max_findings")
    if maximum is not None and (type(maximum) is not int or maximum < 0):
        raise ContractError(f"{label} key report.max_findings must be a non-negative integer")
    return result


def _manifest_defaults(document: Any) -> dict[str, Any]:
    """Extract and validate only the configuration defaults from extension metadata."""
    if type(document) is not dict:
        raise ContractError("extension manifest root must be a mapping")
    return _validate_document(document.get("defaults"), "extension manifest defaults")


def _leaf(document: dict[str, Any], section: str, name: str) -> Any:
    """Return one meaningful leaf, treating null and empty scalars as absent."""
    value = document.get(section, {}).get(name)
    if type(value) is str and value == "":
        return None
    return value


def _first_value(*values: Any) -> Any:
    """Return the first non-absent precedence value."""
    for value in values:
        if value is not None and value != "":
            return value
    return None


def _environment_integer(name: str) -> int | None:
    """Parse a strict non-negative integer environment override."""
    value = os.environ.get(name)
    if value is None or value == "":
        return None
    if not value.isascii() or not value.isdigit():
        raise ContractError(f"environment variable {name} must be a non-negative integer")
    return int(value)


def _environment_globs() -> list[str] | None:
    """Parse the PRD environment override as exactly one strict CSV record."""
    value = os.environ.get("SPECKIT_DIAGRAM_ROADMAP_PRD_GLOBS")
    if value is None or value == "":
        return None
    try:
        records = list(csv.reader(io.StringIO(value), strict=True))
    except csv.Error as error:
        raise ContractError("environment PRD globs must be one valid CSV record") from error
    if len(records) != 1:
        raise ContractError("environment PRD globs must contain exactly one CSV record")
    globs = [item.strip() for item in records[0]]
    if not globs or any(not item for item in globs):
        raise ContractError("environment PRD globs must not contain empty members")
    return globs


def _reject_lexically(value: str, label: str) -> None:
    """Reject path forms that are outside the project-relative contract."""
    if type(value) is not str or not value:
        raise ContractError(f"{label} must be a non-empty project-relative path")
    if "\x00" in value:
        raise ContractError(f"{label} must not contain NUL")
    if value.startswith("~"):
        raise ContractError(f"{label} must not use home expansion")
    if value.startswith("/") or value.startswith("\\\\") or value.startswith("//"):
        raise ContractError(f"{label} must be project-relative")
    if re.match(r"^[A-Za-z]:[\\/]", value):
        raise ContractError(f"{label} must not use a Windows drive path")
    if ".." in value.split("/"):
        raise ContractError(f"{label} must not contain traversal components")


def _bounded_target(project_root: Path, value: str, label: str) -> tuple[Path, str]:
    """Resolve one project-relative path and return its canonical relative form."""
    _reject_lexically(value, label)
    target = _require_contained(project_root / value, project_root, label)
    relative = target.relative_to(project_root).as_posix()
    if relative == ".":
        raise ContractError(f"{label} must not resolve to the project root")
    return target, relative


def _validate_pattern(project_root: Path, pattern: str) -> str:
    """Validate a glob's literal directory prefix without scanning for matches."""
    _reject_lexically(pattern, "prd.globs item")
    literal_parts: list[str] = []
    for part in pattern.split("/"):
        if any(character in part for character in "*?["):
            break
        literal_parts.append(part)
    if literal_parts:
        _require_contained(project_root.joinpath(*literal_parts), project_root, "prd.globs item")
    return pattern


def _existing_parent_is_directory(target: Path, project_root: Path) -> bool:
    """Return whether the nearest existing parent is a contained directory."""
    parent = target.parent
    while not parent.exists() and parent != project_root:
        parent = parent.parent
    return parent.is_dir() and _require_contained(parent, project_root, "roadmap-write path") == parent


def _validate_concrete_path(project_root: Path, kind: str, value: str) -> dict[str, str]:
    """Validate one concrete roadmap, ADR, or PRD path immediately before access."""
    if kind not in VALIDATION_KINDS:
        raise ContractError(f"validation kind must be one of {sorted(VALIDATION_KINDS)}")
    target, relative = _bounded_target(project_root, value, f"{kind} path")
    if kind in {"roadmap-read", "prd"} and not target.is_file():
        raise ContractError(f"{kind} path must be an existing regular file")
    if kind == "adr" and not target.is_dir():
        raise ContractError("adr path must be an existing directory")
    if kind == "roadmap-write":
        if target.exists() and not target.is_file():
            raise ContractError("roadmap-write path must be a regular file when it exists")
        if not target.exists() and not _existing_parent_is_directory(target, project_root):
            raise ContractError("roadmap-write path must have a contained existing parent directory")
    return {"path": relative}


def validate_concrete_path(project_root: Path, kind: str, value: str) -> dict[str, str]:
    """Validate a concrete project path immediately before access."""
    return _validate_concrete_path(project_root, kind, value)


def _resolve_configuration(project_root: Path, payload_root: Path, yaml: Any) -> dict[str, Any]:
    """Resolve the six-field configuration contract from all precedence sources."""
    configuration_path = (
        project_root / ".specify" / "extensions" / "diagram-roadmap" / "roadmap-config.yml"
    )
    manifest_path = payload_root / "extension.yml"
    configuration_text = _read_fixed_yaml(
        configuration_path,
        project_root,
        "project configuration",
        required=False,
    )
    manifest_text = _read_fixed_yaml(
        manifest_path,
        payload_root,
        "extension manifest",
        required=True,
    )
    configuration = _validate_document(
        _load_yaml(configuration_text, "project configuration", yaml),
        "project configuration",
    )
    manifest = _manifest_defaults(_load_yaml(manifest_text, "extension manifest", yaml))
    roadmap_value = _first_value(
        os.environ.get("SPECKIT_DIAGRAM_ROADMAP_PATH"),
        _leaf(configuration, "roadmap", "path"),
        _leaf(manifest, "roadmap", "path"),
        DEFAULT_ROADMAP_PATH,
    )
    adr_value = _first_value(
        os.environ.get("SPECKIT_DIAGRAM_ROADMAP_ADR_DIR"),
        _leaf(configuration, "adr", "dir"),
        _leaf(manifest, "adr", "dir"),
        DEFAULT_ADR_DIRECTORY,
    )
    prd_values = _first_value(
        _environment_globs(),
        _leaf(configuration, "prd", "globs"),
        _leaf(manifest, "prd", "globs"),
        list(DEFAULT_PRD_GLOBS),
    )
    maximum = _first_value(
        _environment_integer("SPECKIT_DIAGRAM_ROADMAP_MAX_FINDINGS"),
        _leaf(configuration, "report", "max_findings"),
        _leaf(manifest, "report", "max_findings"),
        DEFAULT_MAX_FINDINGS,
    )
    if type(roadmap_value) is not str or type(adr_value) is not str:
        raise ContractError("resolved roadmap and ADR values must be strings")
    if type(prd_values) is not list:
        raise ContractError("resolved PRD globs must be a list")
    if type(maximum) is not int or maximum < 0:
        raise ContractError("resolved max_findings must be a non-negative integer")
    roadmap_target, roadmap_relative = _bounded_target(
        project_root, roadmap_value, "roadmap.path"
    )
    adr_target, adr_relative = _bounded_target(project_root, adr_value, "adr.dir")
    patterns = [_validate_pattern(project_root, pattern) for pattern in prd_values]
    return {
        "roadmap_path": roadmap_relative,
        "roadmap_exists": roadmap_target.is_file(),
        "adr_dir": adr_relative,
        "adr_present": adr_target.is_dir(),
        "prd_globs": patterns,
        "max_findings": maximum,
    }


def resolve_configuration(project_root: Path, payload_root: Path, yaml: Any) -> dict[str, Any]:
    """Resolve the stable six-field configuration contract for a sibling script."""
    return _resolve_configuration(project_root, payload_root, yaml)


def _run(yaml: Any) -> dict[str, Any]:
    """Dispatch default or concrete-path validation mode."""
    project_root, payload_root = _discover_roots()
    if not sys.argv[1:]:
        return _resolve_configuration(project_root, payload_root, yaml)
    if len(sys.argv) == 4 and sys.argv[1] == "--validate-path":
        return _validate_concrete_path(project_root, sys.argv[2], sys.argv[3])
    raise ContractError(
        "usage: load_config.py [--validate-path <roadmap-read|roadmap-write|adr|prd> <path>]"
    )


def _write_result(result: dict[str, Any]) -> None:
    """Write exactly one compact UTF-8 JSON object after complete construction."""
    serialized = json.dumps(result, ensure_ascii=False, separators=(",", ":"))
    sys.stdout.write(serialized + "\n")


def _write_error(error: BaseException) -> None:
    """Write one sanitized diagnostic line without exposing source content."""
    message = str(error) if isinstance(error, ContractError) else "unexpected internal failure"
    escaped = json.dumps(message, ensure_ascii=False)[1:-1]
    sys.stderr.write(f"Error: {escaped}\n")


def main() -> int:
    """Run the loader behind a bounded public failure boundary."""
    try:
        yaml = _establish_runtime()
        result = _run(yaml)
        _write_result(result)
        return 0
    except BaseException as error:
        if isinstance(error, (KeyboardInterrupt, SystemExit)):
            raise
        _write_error(error)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
