mod support;

use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::symlink;
use support::project::ProjectFixture;

#[test]
fn selected_human_diff_invokes_named_tool_with_literal_arguments_and_child_exit_code() {
    let fixture = ProjectFixture::initialized();
    let source = fixture.project_root.join("source file");
    let destination = fixture.home_destination("destination file");
    fs::write(&source, "source").unwrap();
    fs::write(&destination, "destination").unwrap();
    support::copy_complete_metadata(
        &source,
        &destination,
        grip::discovery::model::NodeKind::File,
    );
    let added = fixture.command(&["add", "source file", "~/destination file"]);
    assert!(added.status.success(), "{added:?}");

    let recorder = fixture.write_executable(
        "record-diff.sh",
        "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$GRIP_RECORD\"\nexit 7\n",
    );
    let record = fixture.root.path().join("arguments.txt");
    let mut descriptor = fs::read_to_string(fixture.descriptor_path()).unwrap();
    descriptor.push_str(&format!(
        "\n[diff]\ntool = \"record\"\n\n[difftool.record]\nprogram = \"{}\"\nargs = [\"--literal=$(no-shell)\", \"two words\"]\n",
        recorder.display()
    ));
    fs::write(fixture.descriptor_path(), descriptor).unwrap();

    let output = fixture
        .command_builder(&fixture.project_root)
        .env("GRIP_RECORD", &record)
        .args(["diff", "source file"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(7));
    assert_eq!(
        fs::read_to_string(record)
            .unwrap()
            .lines()
            .collect::<Vec<_>>(),
        vec![
            "--literal=$(no-shell)",
            "two words",
            source.to_str().unwrap(),
            destination.to_str().unwrap(),
        ]
    );
    assert!(output.stdout.is_empty());
}

#[test]
fn verbose_selected_diff_explains_grip_inspection_on_stderr_without_polluting_tool_output() {
    let fixture = ProjectFixture::initialized();
    let source = fixture.project_root.join("source");
    let destination = fixture.home_destination("destination");
    fs::write(&source, "accepted").unwrap();
    fs::write(&destination, "accepted").unwrap();
    support::copy_complete_metadata(
        &source,
        &destination,
        grip::discovery::model::NodeKind::File,
    );
    assert!(
        fixture
            .command(&["add", "source", "~/destination"])
            .status
            .success()
    );
    fs::write(&source, "changed").unwrap();
    support::set_fixture_mode(&source, 0o740);
    support::set_fixture_modified_time(&source, 1_700_000_000, 123_456_789);
    let tool = fixture.write_executable("print-diff.sh", "#!/bin/sh\nprintf 'tool output\\n'\n");
    let mut descriptor = fs::read_to_string(fixture.descriptor_path()).unwrap();
    descriptor.push_str(&format!(
        "\n[diff]\ntool = \"print\"\n[difftool.print]\nprogram = \"{}\"\n",
        tool.display()
    ));
    fs::write(fixture.descriptor_path(), descriptor).unwrap();

    let output = fixture
        .command_builder(&fixture.project_root)
        .args(["diff", "-v", "source"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "tool output\n");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Grip inspection:\n"), "{stderr}");
    assert!(
        stderr.contains("Result: Source changed; destination matches the accepted baseline."),
        "{stderr}"
    );
    assert!(stderr.contains("- permission mode:\n"), "{stderr}");
    assert!(stderr.contains("source     : 0740"), "{stderr}");
    assert!(stderr.contains("baseline   : 0644"), "{stderr}");
    assert!(stderr.contains("destination: 0644"), "{stderr}");
    assert!(stderr.contains("- modification time:\n"), "{stderr}");
    assert!(!stderr.contains("Metadata notes:"), "{stderr}");
    assert!(stderr.contains("- source: filesystem"), "{stderr}");
    assert!(!stderr.contains("for /"), "{stderr}");
}

#[test]
fn environment_override_bypasses_global_configuration_and_json_or_unselected_diff_does_not_launch()
{
    let fixture = ProjectFixture::initialized();
    let source = fixture.project_root.join("source");
    let destination = fixture.home_destination("destination");
    fs::write(&source, "source").unwrap();
    fs::write(&destination, "destination").unwrap();
    support::copy_complete_metadata(
        &source,
        &destination,
        grip::discovery::model::NodeKind::File,
    );
    assert!(
        fixture
            .command(&["add", "source", "~/destination"])
            .status
            .success()
    );
    let recorder = fixture.write_executable(
        "record-diff.sh",
        "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$GRIP_RECORD\"\n",
    );
    let record = fixture.root.path().join("arguments.txt");
    let global = fixture.home_root.join(".grip");
    fs::create_dir(&global).unwrap();
    fs::write(global.join("config.toml"), format!("[diff]\ntool = \"ignored\"\n[difftool.ignored]\nprogram = \"{}\"\nargs = [\"ignored\"]\n", recorder.display())).unwrap();

    let json = fixture
        .command_builder(&fixture.project_root)
        .env("GRIP_RECORD", &record)
        .args(["-o", "json", "diff", "source"])
        .output()
        .unwrap();
    assert!(json.status.success());
    assert!(!record.exists());
    let unselected = fixture
        .command_builder(&fixture.project_root)
        .env("GRIP_RECORD", &record)
        .args(["diff"])
        .output()
        .unwrap();
    assert!(unselected.status.success());
    assert!(!record.exists());

    let output = fixture
        .command_builder(&fixture.project_root)
        .env("GRIP_RECORD", &record)
        .env("GRIP_EXTERNAL_DIFF", &recorder)
        .args(["diff", "source"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(
        fs::read_to_string(record)
            .unwrap()
            .lines()
            .collect::<Vec<_>>(),
        vec![source.to_str().unwrap(), destination.to_str().unwrap()]
    );
}

#[test]
fn signaled_comparison_returns_conventional_signal_exit_code() {
    let fixture = ProjectFixture::initialized();
    let source = fixture.project_root.join("source");
    let destination = fixture.home_destination("destination");
    fs::write(&source, "source").unwrap();
    fs::write(&destination, "destination").unwrap();
    support::copy_complete_metadata(
        &source,
        &destination,
        grip::discovery::model::NodeKind::File,
    );
    assert!(
        fixture
            .command(&["add", "source", "~/destination"])
            .status
            .success()
    );
    let killer = fixture.write_executable("kill-diff.sh", "#!/bin/sh\nkill -TERM $$\n");
    let output = fixture
        .command_builder(&fixture.project_root)
        .env("GRIP_EXTERNAL_DIFF", &killer)
        .args(["diff", "source"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(143));
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("signal 15")
    );
}

#[test]
fn project_profile_overrides_global_tool_arguments_and_survives_mapping_updates() {
    let fixture = ProjectFixture::initialized();
    let source = fixture.project_root.join("source");
    let destination = fixture.home_destination("destination");
    let second_source = fixture.project_root.join("second");
    let second_destination = fixture.home_destination("second-destination");
    fs::write(&source, "source").unwrap();
    fs::write(&destination, "destination").unwrap();
    fs::write(&second_source, "second").unwrap();
    fs::write(&second_destination, "second").unwrap();
    support::copy_complete_metadata(
        &source,
        &destination,
        grip::discovery::model::NodeKind::File,
    );
    support::copy_complete_metadata(
        &second_source,
        &second_destination,
        grip::discovery::model::NodeKind::File,
    );
    assert!(
        fixture
            .command(&["add", "source", "~/destination"])
            .status
            .success()
    );

    let recorder = fixture.write_executable(
        "record-diff.sh",
        "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$GRIP_RECORD\"\n",
    );
    let record = fixture.root.path().join("arguments.txt");
    let global = fixture.home_root.join(".grip");
    fs::create_dir(&global).unwrap();
    fs::write(
        global.join("config.toml"),
        format!(
            "[diff]\ntool = \"shared\"\n\n[difftool.shared]\nprogram = \"{}\"\nargs = [\"global\"]\n",
            recorder.display()
        ),
    )
    .unwrap();
    let mut descriptor = fs::read_to_string(fixture.descriptor_path()).unwrap();
    descriptor.push_str("\n[difftool.shared]\nargs = [\"project\"]\n");
    fs::write(fixture.descriptor_path(), descriptor).unwrap();

    assert!(
        fixture
            .command(&["add", "second", "~/second-destination"])
            .status
            .success()
    );
    let descriptor = fs::read_to_string(fixture.descriptor_path()).unwrap();
    assert!(descriptor.contains("[difftool.shared]"));
    assert!(descriptor.contains("args = [\"project\"]"));

    let output = fixture
        .command_builder(&fixture.project_root)
        .env("GRIP_RECORD", &record)
        .args(["diff", "source"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        fs::read_to_string(record)
            .unwrap()
            .lines()
            .collect::<Vec<_>>(),
        vec![
            "project",
            source.to_str().unwrap(),
            destination.to_str().unwrap()
        ]
    );
}

#[test]
fn malformed_profile_is_deferred_for_json_but_rejected_for_selected_human_diff() {
    let fixture = ProjectFixture::initialized();
    let source = fixture.project_root.join("source");
    let destination = fixture.home_destination("destination");
    fs::write(&source, "source").unwrap();
    fs::write(&destination, "destination").unwrap();
    support::copy_complete_metadata(
        &source,
        &destination,
        grip::discovery::model::NodeKind::File,
    );
    assert!(
        fixture
            .command(&["add", "source", "~/destination"])
            .status
            .success()
    );
    let descriptor = fs::read_to_string(fixture.descriptor_path()).unwrap();
    fs::write(
        fixture.descriptor_path(),
        format!("diff = \"malformed\"\n{descriptor}"),
    )
    .unwrap();

    let json = fixture.command(&["-o", "json", "diff", "source"]);
    assert!(json.status.success(), "{json:?}");
    let human = fixture.command(&["diff", "source"]);
    assert_eq!(human.status.code(), Some(10), "{human:?}");
    assert!(
        String::from_utf8(human.stdout)
            .unwrap()
            .contains("diff must be a table")
    );
}

#[test]
fn unavailable_external_program_is_reported_as_an_operational_error() {
    let fixture = ProjectFixture::initialized();
    let source = fixture.project_root.join("source");
    let destination = fixture.home_destination("destination");
    fs::write(&source, "source").unwrap();
    fs::write(&destination, "destination").unwrap();
    support::copy_complete_metadata(
        &source,
        &destination,
        grip::discovery::model::NodeKind::File,
    );
    assert!(
        fixture
            .command(&["add", "source", "~/destination"])
            .status
            .success()
    );
    let output = fixture
        .command_builder(&fixture.project_root)
        .env("GRIP_EXTERNAL_DIFF", "not-a-real-grip-diff-program")
        .args(["diff", "source"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(20));
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("could not launch external comparison")
    );
}

#[test]
fn destination_selected_mapping_root_is_handed_off_and_unsafe_endpoints_are_not() {
    let fixture = ProjectFixture::initialized();
    let source = fixture.project_root.join("tree");
    let destination = fixture.home_destination("tree");
    fs::create_dir(&source).unwrap();
    fs::create_dir(&destination).unwrap();
    fs::write(source.join("member"), "same").unwrap();
    fs::write(destination.join("member"), "same").unwrap();
    support::copy_complete_metadata(
        &source.join("member"),
        &destination.join("member"),
        grip::discovery::model::NodeKind::File,
    );
    support::copy_complete_metadata(
        &source,
        &destination,
        grip::discovery::model::NodeKind::Directory,
    );
    assert!(fixture.command(&["add", "tree", "~/tree"]).status.success());
    let recorder = fixture.write_executable(
        "record-diff.sh",
        "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$GRIP_RECORD\"\n",
    );
    let record = fixture.root.path().join("arguments.txt");
    let selected = fixture
        .command_builder(&fixture.project_root)
        .env("GRIP_EXTERNAL_DIFF", &recorder)
        .env("GRIP_RECORD", &record)
        .args(["diff", "--destination", "~/tree"])
        .output()
        .unwrap();
    assert!(selected.status.success(), "{selected:?}");
    assert_eq!(
        fs::read_to_string(&record)
            .unwrap()
            .lines()
            .collect::<Vec<_>>(),
        vec![source.to_str().unwrap(), destination.to_str().unwrap()]
    );

    fs::remove_file(&record).unwrap();
    let unsafe_target = fixture.root.path().join("unsafe-target");
    fs::write(&unsafe_target, "unsafe").unwrap();
    fs::remove_file(source.join("member")).unwrap();
    symlink(&unsafe_target, source.join("member")).unwrap();
    let unsafe_diff = fixture
        .command_builder(&fixture.project_root)
        .env("GRIP_EXTERNAL_DIFF", &recorder)
        .env("GRIP_RECORD", &record)
        .args(["diff", "tree/member"])
        .output()
        .unwrap();
    assert!(!unsafe_diff.status.success(), "{unsafe_diff:?}");
    assert!(!record.exists());
}

#[test]
fn default_diff_fallback_uses_path_and_propagates_exit_one() {
    let fixture = ProjectFixture::initialized();
    let source = fixture.project_root.join("source");
    let destination = fixture.home_destination("destination");
    fs::write(&source, "source").unwrap();
    fs::write(&destination, "destination").unwrap();
    support::copy_complete_metadata(
        &source,
        &destination,
        grip::discovery::model::NodeKind::File,
    );
    assert!(
        fixture
            .command(&["add", "source", "~/destination"])
            .status
            .success()
    );
    let fallback = fixture.write_executable(
        "diff",
        "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$GRIP_RECORD\"\nexit 1\n",
    );
    let record = fixture.root.path().join("arguments.txt");

    let output = fixture
        .command_builder(&fixture.project_root)
        .env("PATH", fallback.parent().unwrap())
        .env("GRIP_RECORD", &record)
        .args(["diff", "source"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert_eq!(
        fs::read_to_string(record)
            .unwrap()
            .lines()
            .collect::<Vec<_>>(),
        vec![source.to_str().unwrap(), destination.to_str().unwrap()]
    );
}

#[test]
fn project_tool_selection_overrides_global_and_invalid_global_or_empty_environment_do_not_launch() {
    let fixture = ProjectFixture::initialized();
    let source = fixture.project_root.join("source");
    let destination = fixture.home_destination("destination");
    fs::write(&source, "source").unwrap();
    fs::write(&destination, "destination").unwrap();
    support::copy_complete_metadata(
        &source,
        &destination,
        grip::discovery::model::NodeKind::File,
    );
    assert!(
        fixture
            .command(&["add", "source", "~/destination"])
            .status
            .success()
    );
    let global_program = fixture.write_executable(
        "global-diff.sh",
        "#!/bin/sh\nprintf global > \"$GRIP_RECORD\"\n",
    );
    let project_program = fixture.write_executable(
        "project-diff.sh",
        "#!/bin/sh\nprintf project > \"$GRIP_RECORD\"\n",
    );
    let record = fixture.root.path().join("arguments.txt");
    let global = fixture.home_root.join(".grip");
    fs::create_dir(&global).unwrap();
    fs::write(
        global.join("config.toml"),
        format!(
            "[diff]\ntool = \"global\"\n[difftool.global]\nprogram = \"{}\"\n",
            global_program.display()
        ),
    )
    .unwrap();
    let mut descriptor = fs::read_to_string(fixture.descriptor_path()).unwrap();
    descriptor.push_str(&format!(
        "\n[diff]\ntool = \"project\"\n[difftool.project]\nprogram = \"{}\"\n",
        project_program.display()
    ));
    fs::write(fixture.descriptor_path(), descriptor).unwrap();

    let selected = fixture
        .command_builder(&fixture.project_root)
        .env("GRIP_RECORD", &record)
        .args(["diff", "source"])
        .output()
        .unwrap();
    assert!(selected.status.success(), "{selected:?}");
    assert_eq!(fs::read_to_string(&record).unwrap(), "project");

    fs::remove_file(&record).unwrap();
    fs::write(global.join("config.toml"), "diff = \"malformed\"\n").unwrap();
    let malformed = fixture
        .command_builder(&fixture.project_root)
        .env("GRIP_RECORD", &record)
        .args(["diff", "source"])
        .output()
        .unwrap();
    assert_eq!(malformed.status.code(), Some(10), "{malformed:?}");
    assert!(!record.exists());

    let empty_override = fixture
        .command_builder(&fixture.project_root)
        .env("GRIP_RECORD", &record)
        .env("GRIP_EXTERNAL_DIFF", "")
        .args(["diff", "source"])
        .output()
        .unwrap();
    assert_eq!(empty_override.status.code(), Some(10), "{empty_override:?}");
    assert!(!record.exists());
}

#[test]
fn missing_or_unsupported_selected_endpoint_never_launches_the_program() {
    let fixture = ProjectFixture::initialized();
    let source = fixture.project_root.join("source");
    let destination = fixture.home_destination("destination");
    fs::write(&source, "source").unwrap();
    fs::write(&destination, "destination").unwrap();
    support::copy_complete_metadata(
        &source,
        &destination,
        grip::discovery::model::NodeKind::File,
    );
    assert!(
        fixture
            .command(&["add", "source", "~/destination"])
            .status
            .success()
    );
    let recorder =
        fixture.write_executable("record-diff.sh", "#!/bin/sh\ntouch \"$GRIP_RECORD\"\n");
    let record = fixture.root.path().join("external-diff-ran");

    fs::remove_file(&source).unwrap();
    let missing = fixture
        .command_builder(&fixture.project_root)
        .env("GRIP_EXTERNAL_DIFF", &recorder)
        .env("GRIP_RECORD", &record)
        .args(["diff", "source"])
        .output()
        .unwrap();
    assert!(
        String::from_utf8(missing.stdout)
            .unwrap()
            .contains("source_side_deletion")
    );
    assert!(!record.exists());

    let fifo_name = std::ffi::CString::new(source.as_os_str().as_bytes()).unwrap();
    assert_eq!(unsafe { libc::mkfifo(fifo_name.as_ptr(), 0o600) }, 0);
    let unsupported = fixture
        .command_builder(&fixture.project_root)
        .env("GRIP_EXTERNAL_DIFF", &recorder)
        .env("GRIP_RECORD", &record)
        .args(["diff", "source"])
        .output()
        .unwrap();
    assert!(!unsupported.stdout.is_empty());
    assert!(!record.exists());
}
