mod support;

use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

fn inspect(root: &Path, source: &Path, policy: &str) -> BTreeMap<String, String> {
    let grip_home = support::minimal_home(root);
    let destination = root.join("destination");
    fs::write(source.join(".gripignore"), policy).unwrap();
    support::write_registry(&grip_home, &[("tree", source, &destination)]);
    let output = support::command_with_grip_home(
        root,
        &grip_home,
        &[
            "--output",
            "json",
            "mapping",
            "inspect",
            source.to_str().unwrap(),
        ],
    );
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    result["details"]["records"]
        .as_array()
        .unwrap()
        .iter()
        .map(|record| {
            (
                record["relative_path"]["display"]
                    .as_str()
                    .unwrap()
                    .to_owned(),
                record["category"].as_str().unwrap().to_owned(),
            )
        })
        .collect()
}

#[test]
fn root_policy_supports_gitignore_grammar_and_policy_only_exclusion() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("source");
    fs::create_dir_all(source.join("cache")).unwrap();
    fs::create_dir_all(source.join("deep/a")).unwrap();
    fs::create_dir_all(source.join("prefix")).unwrap();
    fs::create_dir_all(source.join("x")).unwrap();
    fs::create_dir_all(source.join("a/x")).unwrap();
    for path in [
        "drop.tmp",
        "keep.tmp",
        "root-only",
        "file1.txt",
        "a.log",
        "#literal",
        "!literal",
        "cache/hidden.txt",
        "deep/a/target",
        "prefix/child",
        "x/anywhere",
        "a/x/b",
        "trimmed",
        "literal ",
        ".hidden",
    ] {
        fs::write(source.join(path), "x").unwrap();
    }
    let records = inspect(
        root.path(),
        &source,
        "\u{feff}# comment\r\n\r\n*.tmp\r\n!keep.tmp\r\n/root-only\r\nfile?.txt\r\n[ab].log\r\n\\#literal\r\n\\!literal\r\ncache/\r\ndeep/**/target\r\nprefix/**\r\n**/anywhere\r\na/**/b\r\ntrimmed   \r\nliteral\\ ",
    );
    for ignored in [
        "drop.tmp",
        "root-only",
        "file1.txt",
        "a.log",
        "#literal",
        "!literal",
        "cache",
        "deep/a/target",
        "prefix/child",
        "x/anywhere",
        "a/x/b",
        "trimmed",
        "literal ",
    ] {
        assert_eq!(
            records.get(ignored).map(String::as_str),
            Some("ignored"),
            "{ignored}"
        );
    }
    assert_eq!(
        records.get("keep.tmp").map(String::as_str),
        Some("eligible")
    );
    assert_eq!(records.get(".hidden").map(String::as_str), Some("eligible"));
    assert!(!records.contains_key("cache/hidden.txt"));
    assert!(!records.contains_key(".gripignore"));
}

#[test]
fn nested_policy_has_deeper_precedence_but_cannot_reinclude_below_a_pruned_parent() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("source");
    fs::create_dir_all(source.join("nested")).unwrap();
    fs::create_dir_all(source.join("pruned")).unwrap();
    fs::write(source.join("nested/keep.txt"), "keep").unwrap();
    fs::write(source.join("nested/drop.txt"), "drop").unwrap();
    fs::write(source.join("nested/.gripignore"), "!keep.txt\n").unwrap();
    fs::write(source.join("pruned/hidden.txt"), "hidden").unwrap();
    fs::write(source.join("pruned/.gripignore"), "!hidden.txt\n").unwrap();
    fs::write(source.join(".gitignore"), "nested/keep.txt\n").unwrap();
    fs::write(source.join(".ignore"), "nested/keep.txt\n").unwrap();
    fs::set_permissions(source.join("pruned"), fs::Permissions::from_mode(0o000)).unwrap();
    let records = inspect(root.path(), &source, "*.txt\npruned/\n!.gripignore\n");
    fs::set_permissions(source.join("pruned"), fs::Permissions::from_mode(0o700)).unwrap();
    assert_eq!(
        records.get("nested/keep.txt").map(String::as_str),
        Some("eligible")
    );
    assert_eq!(
        records.get("nested/drop.txt").map(String::as_str),
        Some("ignored")
    );
    assert_eq!(records.get("pruned").map(String::as_str), Some("ignored"));
    assert!(!records.contains_key("pruned/hidden.txt"));
    assert!(!records.contains_key("nested/.gripignore"));
    assert!(!records.contains_key("pruned/.gripignore"));
    assert_eq!(
        records.get(".gitignore").map(String::as_str),
        Some("eligible")
    );
    assert_eq!(records.get(".ignore").map(String::as_str), Some("eligible"));
}
