mod support;

use std::fs;
use std::os::unix::fs::symlink;
use support::project::ContainedHomeFixture;

fn json(output: &std::process::Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "invalid JSON: {error}; stdout={}; stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

#[test]
fn contained_home_mapping_adds_without_payload_mutation_and_pushes_safe_members() {
    let fixture = ContainedHomeFixture::initialized();
    fs::create_dir_all(fixture.source_root.join(".local/bin")).unwrap();
    fs::write(fixture.source_root.join(".bashrc"), "export EDITOR=vi\n").unwrap();
    fs::write(fixture.source_root.join(".gitconfig"), "[user]\n").unwrap();
    fs::write(fixture.source_root.join(".local/bin/tool"), "tool\n").unwrap();
    let before = fixture.snapshot_payloads();

    let add = fixture.command(&["--output=json", "add", "home/", "~/"]);
    assert!(
        add.status.success(),
        "{}",
        String::from_utf8_lossy(&add.stderr)
    );
    assert_eq!(fixture.snapshot_payloads(), before);

    let status = fixture.command(&["--output=json", "status"]);
    assert!(
        status.status.success(),
        "{}",
        String::from_utf8_lossy(&status.stderr)
    );
    let status = json(&status);
    let records = status["details"]["records"].as_array().unwrap();
    assert_eq!(records.len(), 5);
    assert!(records.iter().all(|record| {
        !record["destination_path"]
            .as_str()
            .unwrap_or_default()
            .contains("grip-project")
    }));

    let preview = fixture.command(&["--output=json", "push", "--dry-run"]);
    assert!(
        preview.status.success(),
        "{}",
        String::from_utf8_lossy(&preview.stderr)
    );
    assert!(!fixture.home_root.join(".bashrc").exists());

    let push = fixture.command(&["--output=json", "push"]);
    assert!(
        push.status.success(),
        "{}",
        String::from_utf8_lossy(&push.stderr)
    );
    assert_eq!(
        fs::read(fixture.home_root.join(".bashrc")).unwrap(),
        b"export EDITOR=vi\n"
    );
    assert_eq!(
        fs::read(fixture.home_root.join(".local/bin/tool")).unwrap(),
        b"tool\n"
    );

    fs::write(fixture.home_root.join(".bashrc"), "export EDITOR=nvim\n").unwrap();
    let pull_preview = fixture.command(&["--output=json", "pull", "--dry-run"]);
    assert!(pull_preview.status.success());
    assert_eq!(
        fs::read(fixture.source_root.join(".bashrc")).unwrap(),
        b"export EDITOR=vi\n"
    );
    assert!(fixture.command(&["pull"]).status.success());
    assert_eq!(
        fs::read(fixture.source_root.join(".bashrc")).unwrap(),
        b"export EDITOR=nvim\n"
    );

    fs::write(
        fixture.source_root.join(".gitconfig"),
        "[user]\nname = Example\n",
    )
    .unwrap();
    fs::write(
        fixture.home_root.join(".local/bin/tool"),
        "destination tool\n",
    )
    .unwrap();
    let sync_preview = fixture.command(&["--output=json", "sync", "--dry-run"]);
    assert!(sync_preview.status.success());
    assert!(fixture.command(&["sync"]).status.success());
    assert_eq!(
        fs::read(fixture.home_root.join(".gitconfig")).unwrap(),
        b"[user]\nname = Example\n"
    );
    assert_eq!(
        fs::read(fixture.source_root.join(".local/bin/tool")).unwrap(),
        b"destination tool\n"
    );
}

#[test]
fn contained_home_add_admits_an_exact_destination_link_member_without_touching_its_target() {
    let fixture = ContainedHomeFixture::initialized();
    let source = fixture.source_root.join(".bashrc");
    let target = fixture.root.path().join("target");
    let destination = fixture.home_root.join(".bashrc");
    fs::write(&source, "managed\n").unwrap();
    fs::write(&target, "unchanged\n").unwrap();
    symlink(&target, &destination).unwrap();

    let add = fixture.command(&["--output=json", "add", "home/", "~/"]);
    assert!(
        add.status.success(),
        "{}",
        String::from_utf8_lossy(&add.stdout)
    );
    assert!(
        fs::symlink_metadata(&destination)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert_eq!(fs::read_to_string(&target).unwrap(), "unchanged\n");
    let status = json(&fixture.command(&["--output=json", "status"]));
    assert!(
        status["details"]["records"]
            .as_array()
            .unwrap()
            .iter()
            .any(|record| {
                record["relative_path"]["display"] == ".bashrc"
                    && record["classification"] == "unresolved_destination_link"
            })
    );
    let human_status = fixture.command(&["status"]);
    assert!(human_status.status.success());
    let human_status = String::from_utf8(human_status.stdout).unwrap();
    assert!(human_status.contains("Replace destination link: grip push --force home/.bashrc"));
    assert!(!human_status.contains("grip pull --force"));

    let dry_run = fixture.command(&["push", "--dry-run"]);
    assert!(!dry_run.status.success());
    let dry_run = String::from_utf8(dry_run.stdout).unwrap();
    assert!(dry_run.contains("Replace destination link: grip push --force home/.bashrc"));
    assert!(!dry_run.contains("grip pull --force"));
}

#[test]
fn forced_push_replaces_an_empty_directory_destination_link_without_touching_its_target() {
    let fixture = ContainedHomeFixture::initialized();
    let source = fixture.source_root.join(".config");
    let target = fixture.root.path().join("former-directory-target");
    let destination = fixture.home_root.join(".config");
    fs::create_dir(&source).unwrap();
    fs::create_dir(&target).unwrap();
    fs::write(target.join("sentinel"), "unchanged\n").unwrap();
    symlink(&target, &destination).unwrap();
    let added = fixture.command(&["add", "home/", "~/"]);
    assert!(
        added.status.success(),
        "{}",
        String::from_utf8_lossy(&added.stdout)
    );

    let pushed = fixture.command(&["--output=json", "push", "--force", "home/.config"]);
    assert!(
        pushed.status.success(),
        "{}",
        String::from_utf8_lossy(&pushed.stdout)
    );
    assert!(fs::symlink_metadata(&destination).unwrap().is_dir());
    assert!(
        !fs::symlink_metadata(&destination)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert_eq!(
        fs::read_to_string(target.join("sentinel")).unwrap(),
        "unchanged\n"
    );
}

#[test]
fn forced_push_replaces_a_directory_destination_link_with_its_selected_contents() {
    let fixture = ContainedHomeFixture::initialized();
    let source = fixture.source_root.join(".config");
    let target = fixture.root.path().join("former-directory-target");
    let destination = fixture.home_root.join(".config");
    fs::create_dir(&source).unwrap();
    fs::write(source.join("settings"), "managed\n").unwrap();
    fs::create_dir(&target).unwrap();
    fs::write(target.join("sentinel"), "unchanged\n").unwrap();
    symlink(&target, &destination).unwrap();
    let added = fixture.command(&["add", "home/", "~/"]);
    assert!(
        added.status.success(),
        "{}",
        String::from_utf8_lossy(&added.stdout)
    );

    let pushed = fixture.command(&["--output=json", "push", "--force", "home/.config"]);
    assert!(
        pushed.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&pushed.stdout),
        String::from_utf8_lossy(&pushed.stderr)
    );
    assert_eq!(
        fs::read_to_string(destination.join("settings")).unwrap(),
        "managed\n"
    );
    assert_eq!(
        fs::read_to_string(target.join("sentinel")).unwrap(),
        "unchanged\n"
    );
}

#[test]
fn unrelated_destination_breadth_is_not_inventory() {
    let fixture = ContainedHomeFixture::initialized();
    fs::write(fixture.source_root.join(".bashrc"), "managed\n").unwrap();
    let unrelated = fixture.home_root.join("unrelated");
    fs::create_dir(&unrelated).unwrap();
    for index in 0..10_000 {
        fs::write(unrelated.join(format!("item-{index:05}")), b"").unwrap();
    }
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&unrelated, fs::Permissions::from_mode(0o000)).unwrap();
    std::os::unix::fs::symlink(&unrelated, fixture.home_root.join("unrelated-link")).unwrap();

    let add = fixture.command(&["--output=json", "add", "home/", "~/"]);
    assert!(
        add.status.success(),
        "{}",
        String::from_utf8_lossy(&add.stderr)
    );
    let status = json(&fixture.command(&["--output=json", "status"]));
    let rendered = serde_json::to_string(&status).unwrap();
    assert!(!rendered.contains("item-00000"));
    assert!(!rendered.contains("unrelated-link"));
    fs::set_permissions(&unrelated, fs::Permissions::from_mode(0o700)).unwrap();
}

#[test]
fn recursive_current_member_blocks_with_relation_and_ignore_is_remediation() {
    let fixture = ContainedHomeFixture::initialized();
    fs::create_dir_all(fixture.source_root.join("grip-project/home")).unwrap();
    fs::write(
        fixture.source_root.join("grip-project/home/unsafe"),
        "unsafe\n",
    )
    .unwrap();
    let before = fixture.snapshot_payloads();

    let blocked = fixture.command(&["--output=json", "add", "home/", "~/"]);
    assert!(!blocked.status.success());
    let rendered = String::from_utf8_lossy(&blocked.stdout);
    assert!(rendered.contains("recursive_member_topology"), "{rendered}");
    assert!(
        rendered.contains("ancestor") || rendered.contains("equal"),
        "{rendered}"
    );
    assert_eq!(fixture.snapshot_payloads(), before);

    fs::write(fixture.source_root.join(".gripignore"), "grip-project/\n").unwrap();
    let accepted = fixture.command(&["--output=json", "add", "home/", "~/"]);
    assert!(
        accepted.status.success(),
        "{}",
        String::from_utf8_lossy(&accepted.stderr)
    );
}

#[test]
fn retained_source_deletion_is_classified_without_destination_enumeration() {
    let fixture = ContainedHomeFixture::initialized();
    let source = fixture.source_root.join(".gitconfig");
    fs::write(&source, "accepted\n").unwrap();
    assert!(fixture.command(&["add", "home/", "~/"]).status.success());
    assert!(fixture.command(&["push"]).status.success());
    fs::remove_file(&source).unwrap();
    fs::create_dir(fixture.home_root.join("unrelated")).unwrap();
    fs::write(fixture.home_root.join("unrelated/noise"), "noise\n").unwrap();

    let status = fixture.command(&["--output=json", "status"]);
    assert!(status.status.success());
    let status = json(&status);
    let records = status["details"]["records"].as_array().unwrap();
    assert!(records.iter().any(|record| {
        record["relative_path"]["display"] == ".gitconfig"
            && record["classification"] == "source_side_deletion"
    }));
    assert!(
        !serde_json::to_string(&status)
            .unwrap()
            .contains("unrelated/noise")
    );
}

#[test]
fn destination_link_ancestor_blocks_a_descendant_selector() {
    let fixture = ContainedHomeFixture::initialized();
    fs::create_dir_all(fixture.source_root.join(".local/bin")).unwrap();
    fs::write(fixture.source_root.join(".local/bin/tool"), "tool\n").unwrap();
    let target = fixture.root.path().join("former-local-target");
    fs::create_dir(&target).unwrap();
    fs::write(target.join("sentinel"), "unchanged\n").unwrap();
    std::os::unix::fs::symlink(&target, fixture.home_root.join(".local")).unwrap();

    let add = fixture.command(&["--output=json", "add", "home/", "~/"]);
    assert!(
        add.status.success(),
        "{}",
        String::from_utf8_lossy(&add.stdout)
    );
    let forced_descendant = fixture.command(&["push", "--force", "home/.local/bin/tool"]);
    assert!(!forced_descendant.status.success());
    assert!(
        fs::symlink_metadata(fixture.home_root.join(".local"))
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert_eq!(
        fs::read_to_string(target.join("sentinel")).unwrap(),
        "unchanged\n"
    );
}

#[test]
fn exact_selection_cannot_bypass_an_unsafe_sibling_in_the_mapping() {
    let fixture = ContainedHomeFixture::initialized();
    fs::write(fixture.source_root.join(".bashrc"), "safe\n").unwrap();
    assert!(fixture.command(&["add", "home/", "~/"]).status.success());
    fs::create_dir_all(fixture.source_root.join("grip-project/home")).unwrap();
    fs::write(
        fixture.source_root.join("grip-project/home/unsafe"),
        "unsafe\n",
    )
    .unwrap();

    let push = fixture.command(&["--output=json", "push", "home/.bashrc"]);
    assert!(!push.status.success());
    assert!(String::from_utf8_lossy(&push.stdout).contains("recursive_member_topology"));
    assert!(!fixture.home_root.join(".bashrc").exists());
}
