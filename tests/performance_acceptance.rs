mod support;
use std::process::Command;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
struct Distribution {
    p50: Duration,
    p95: Duration,
    maximum: Duration,
}

fn distribution(mut samples: Vec<Duration>) -> Distribution {
    samples.sort_unstable();
    let percentile = |value: usize| {
        let index = (samples.len() * value).div_ceil(100).saturating_sub(1);
        samples[index]
    };
    Distribution {
        p50: percentile(50),
        p95: percentile(95),
        maximum: *samples.last().unwrap(),
    }
}
fn sample_count() -> usize {
    std::env::var("GRIP_PERFORMANCE_SAMPLE_COUNT")
        .ok()
        .and_then(|value| value.parse().ok())
        .filter(|count| *count > 0)
        .unwrap_or(100)
}
fn measure(mut command: impl FnMut() -> Command) -> Distribution {
    let _ = command().output().unwrap();
    let samples = (0..sample_count())
        .map(|_| {
            let start = Instant::now();
            let output = command().output().unwrap();
            assert!(output.status.success());
            start.elapsed()
        })
        .collect();
    distribution(samples)
}

fn measure_consistent(mut command: impl FnMut() -> Command) -> Distribution {
    let expected = command().output().unwrap();
    let samples = (0..sample_count())
        .map(|_| {
            let start = Instant::now();
            let output = command().output().unwrap();
            let elapsed = start.elapsed();
            assert_eq!(output.status.code(), expected.status.code());
            assert_eq!(output.stdout, expected.stdout);
            assert_eq!(output.stderr, expected.stderr);
            elapsed
        })
        .collect();
    distribution(samples)
}

fn command_output(program: &str, arguments: &[&str]) -> String {
    String::from_utf8(
        Command::new(program)
            .args(arguments)
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap()
    .trim()
    .to_owned()
}

#[test]
#[ignore = "representative-workstation acceptance harness"]
fn warm_release_commands_meet_p95_targets() {
    let binary = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/release/grip");
    assert!(binary.is_file(), "build release binary first");
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let registry_project = root.path().join("registry-project");
    std::fs::create_dir(&registry_project).unwrap();
    let metadata_dir = support::initialize_project_metadata(&registry_project);
    let source_root = registry_project.join("sources");
    let destination_root = registry_project.join("destinations");
    std::fs::create_dir(&source_root).unwrap();
    std::fs::create_dir(&destination_root).unwrap();
    let mappings = (0..1_000)
        .map(|index| {
            let source = source_root.join(format!("entry-{index:04}"));
            std::fs::write(&source, index.to_string()).unwrap();
            (
                "file",
                source,
                destination_root.join(format!("entry-{index:04}")),
            )
        })
        .collect::<Vec<_>>();
    let borrowed = mappings
        .iter()
        .map(|(kind, source, destination)| (*kind, source.as_path(), destination.as_path()))
        .collect::<Vec<_>>();
    support::write_descriptor(&metadata_dir, &borrowed);
    eprintln!(
        "os={} arch={} host={} revision={} rust={} profile=release runs={}",
        std::env::consts::OS,
        std::env::consts::ARCH,
        command_output("uname", &["-n"]),
        command_output("git", &["rev-parse", "HEAD"]),
        env!("CARGO_PKG_RUST_VERSION"),
        sample_count()
    );
    let help = measure(|| {
        let mut c = Command::new(&binary);
        c.env_clear()
            .env("HOME", &registry_project)
            .current_dir(&registry_project)
            .arg("--help");
        c
    });
    let version = measure(|| {
        let mut c = Command::new(&binary);
        c.env_clear()
            .env("HOME", &registry_project)
            .current_dir(&registry_project)
            .arg("version");
        c
    });
    let validate = measure(|| {
        let mut c = Command::new(&binary);
        c.env_clear()
            .env("HOME", &registry_project)
            .current_dir(&registry_project)
            .arg("validate");
        c
    });
    let mapping_list = measure_consistent(|| {
        let mut c = Command::new(&binary);
        c.env_clear()
            .env("HOME", &registry_project)
            .current_dir(&registry_project)
            .args(["--output=json", "mapping", "list"]);
        c
    });
    let discovery_project = root.path().join("discovery-project");
    std::fs::create_dir(&discovery_project).unwrap();
    let discovery_home = support::initialize_project_metadata(&discovery_project);
    let discovery_source = discovery_project.join("sources");
    let discovery_destination = discovery_project.join("destinations");
    std::fs::create_dir(&discovery_source).unwrap();
    std::fs::create_dir(&discovery_destination).unwrap();
    let mut deep_directory = discovery_project.clone();
    for _ in 0..100 {
        deep_directory.push("d");
    }
    std::fs::create_dir_all(&deep_directory).unwrap();
    let qualification = support::qualification_record(&discovery_source, "release");
    eprintln!(
        "qualification={}",
        serde_json::to_string(&qualification).unwrap()
    );
    for directory_index in 0..100 {
        let relative_directory = format!("directory-{directory_index:03}");
        let directory = discovery_source.join(&relative_directory);
        let destination_directory = discovery_destination.join(&relative_directory);
        std::fs::create_dir(&directory).unwrap();
        std::fs::create_dir(&destination_directory).unwrap();
        for file_index in 33..99 {
            let name = format!("entry-{file_index:03}.txt");
            let source_file = directory.join(&name);
            let destination_file = destination_directory.join(name);
            std::fs::write(&source_file, b"representative").unwrap();
            std::fs::write(&destination_file, b"representative").unwrap();
            let seconds = 1_700_000_000 + i64::from(directory_index * 100 + file_index);
            support::set_fixture_modified_time(&source_file, seconds, 123_456_789);
            support::set_fixture_modified_time(&destination_file, seconds, 123_456_789);
        }
        support::set_fixture_modified_time(&directory, 1_700_100_000, 987_654_321);
        support::set_fixture_modified_time(&destination_directory, 1_700_100_000, 987_654_321);
    }
    for (name, value) in [
        ("com.apple.TextEncoding", b"utf-8".as_slice()),
        (
            "com.apple.ResourceFork",
            b"representative-resource-fork".as_slice(),
        ),
    ] {
        support::set_fixture_xattr(
            &discovery_source.join("directory-000/entry-033.txt"),
            name,
            value,
        );
        support::set_fixture_xattr(
            &discovery_destination.join("directory-000/entry-033.txt"),
            name,
            value,
        );
    }
    support::set_fixture_mode(&discovery_source.join("directory-001/entry-033.txt"), 0o640);
    support::set_fixture_mode(
        &discovery_destination.join("directory-001/entry-033.txt"),
        0o640,
    );
    support::set_fixture_bsd_flags(
        &discovery_source.join("directory-002/entry-033.txt"),
        libc::UF_NODUMP,
    );
    support::set_fixture_bsd_flags(
        &discovery_destination.join("directory-002/entry-033.txt"),
        libc::UF_NODUMP,
    );
    for directory_index in 0..3 {
        let seconds = 1_700_000_000 + i64::from(directory_index * 100 + 33);
        support::set_fixture_modified_time(
            &discovery_source.join(format!("directory-{directory_index:03}/entry-033.txt")),
            seconds,
            123_456_789,
        );
        support::set_fixture_modified_time(
            &discovery_destination.join(format!("directory-{directory_index:03}/entry-033.txt")),
            seconds,
            123_456_789,
        );
    }
    std::fs::write(
        discovery_source.join("directory-050/.gripignore"),
        "*.ignored\n",
    )
    .unwrap();
    support::set_fixture_modified_time(
        &discovery_source.join("directory-050"),
        1_700_100_000,
        987_654_321,
    );
    support::write_descriptor(
        &discovery_home,
        &[("tree", &discovery_source, &discovery_destination)],
    );
    let implicit_discovery_100_levels = measure_consistent(|| {
        let mut command = Command::new(&binary);
        command
            .env_clear()
            .env("HOME", &discovery_project)
            .current_dir(&deep_directory)
            .args(["--output=json", "mapping", "list"]);
        command
    });
    let discovery = measure_consistent(|| {
        let mut command = Command::new(&binary);
        command
            .env_clear()
            .env("HOME", &discovery_project)
            .current_dir(&discovery_project)
            .args(["--output=json", "mapping", "inspect"])
            .arg(&discovery_source);
        command
    });
    let accepted = Command::new(&binary)
        .env_clear()
        .env("HOME", &discovery_project)
        .current_dir(&discovery_project)
        .args(["--output=json", "baseline", "accept"])
        .output()
        .unwrap();
    assert!(
        accepted.status.success(),
        "{}{}",
        String::from_utf8_lossy(&accepted.stdout),
        String::from_utf8_lossy(&accepted.stderr)
    );
    for directory_index in 0..100 {
        let source_directory = discovery_source.join(format!("directory-{directory_index:03}"));
        for file_index in 0..33 {
            std::fs::write(
                source_directory.join(format!("entry-{file_index:03}.txt")),
                b"source addition",
            )
            .unwrap();
        }
        for file_index in 33..66 {
            std::fs::write(
                source_directory.join(format!("entry-{file_index:03}.txt")),
                b"source replacement",
            )
            .unwrap();
        }
    }
    let status = measure_consistent(|| {
        let mut command = Command::new(&binary);
        command
            .env_clear()
            .env("HOME", &discovery_project)
            .current_dir(&discovery_project)
            .args(["--output=json", "status"]);
        command
    });
    let dry_run_push = measure_consistent(|| {
        let mut command = Command::new(&binary);
        command
            .env_clear()
            .env("HOME", &discovery_project)
            .current_dir(&discovery_project)
            .args(["--output=json", "push", "--dry-run"]);
        command
    });
    let home = support::project_home(&discovery_home);
    let registry = grip::registry::publication::load(&home, false).unwrap();
    let state = grip::state::publication::load(&home).unwrap();
    let selection = grip::observation::model::Selection::All;
    let build_plan = || {
        let observed =
            grip::observation::inspect(&home, &registry, &state.accepted, &selection).unwrap();
        let records = observed
            .values()
            .map(|entry| grip::classification::classify_accepted(entry, &state.accepted))
            .collect();
        grip::push::plan::build_with_parent_requirements(
            grip::classification::model::ClassificationScope {
                kind: "all".into(),
                path_space: grip::observation::model::PathSpace::Source,
                selector: None,
                mapping_source: None,
            },
            records,
            registry.missing_destination_parents(),
        )
        .unwrap()
    };
    let expected_plan = build_plan();
    let execute_mode_plan = distribution(
        (0..sample_count())
            .map(|_| {
                let start = Instant::now();
                assert_eq!(build_plan(), expected_plan);
                start.elapsed()
            })
            .collect(),
    );
    let dry_run_output = Command::new(&binary)
        .env_clear()
        .env("HOME", &discovery_project)
        .current_dir(&discovery_project)
        .args(["--output=json", "push", "--dry-run"])
        .output()
        .unwrap();
    assert_eq!(
        support::json(&dry_run_output)["details"]["plan_id"]["digest"],
        expected_plan.plan_id
    );
    assert_eq!(expected_plan.counts.actionable, 6_700);

    for directory_index in 0..100 {
        let source_directory = discovery_source.join(format!("directory-{directory_index:03}"));
        let destination_directory =
            discovery_destination.join(format!("directory-{directory_index:03}"));
        for file_index in 0..66 {
            let source_file = source_directory.join(format!("entry-{file_index:03}.txt"));
            let destination_file = destination_directory.join(format!("entry-{file_index:03}.txt"));
            std::fs::copy(&source_file, &destination_file).unwrap();
            support::copy_complete_metadata(
                &source_file,
                &destination_file,
                grip::discovery::model::NodeKind::File,
            );
        }
        support::copy_complete_metadata(
            &source_directory,
            &destination_directory,
            grip::discovery::model::NodeKind::Directory,
        );
    }
    let pull_accepted = Command::new(&binary)
        .env_clear()
        .env("HOME", &discovery_project)
        .current_dir(&discovery_project)
        .args(["baseline", "accept"])
        .output()
        .unwrap();
    assert!(pull_accepted.status.success());
    for directory_index in 0..100 {
        let destination_directory =
            discovery_destination.join(format!("directory-{directory_index:03}"));
        for file_index in 0..33 {
            std::fs::write(
                destination_directory.join(format!("entry-{file_index:03}.txt")),
                b"destination replacement",
            )
            .unwrap();
        }
    }
    let dry_run_pull = measure_consistent(|| {
        let mut command = Command::new(&binary);
        command
            .env_clear()
            .env("HOME", &discovery_project)
            .current_dir(&discovery_project)
            .args(["--output=json", "pull", "--dry-run"]);
        command
    });
    let pull_state = grip::state::publication::load(&home).unwrap();
    let build_pull_plan = || {
        let observed =
            grip::observation::inspect(&home, &registry, &pull_state.accepted, &selection).unwrap();
        let records = observed
            .values()
            .map(|entry| grip::classification::classify_accepted(entry, &pull_state.accepted))
            .collect();
        grip::mutation::plan::build_for(
            grip::mutation::model::MutationDirection::Pull,
            grip::classification::model::ClassificationScope {
                kind: "all".into(),
                path_space: grip::observation::model::PathSpace::Source,
                selector: None,
                mapping_source: None,
            },
            records,
        )
        .unwrap()
    };
    let expected_pull_plan = build_pull_plan();
    let pull_execute_plan = distribution(
        (0..sample_count())
            .map(|_| {
                let start = Instant::now();
                assert_eq!(build_pull_plan(), expected_pull_plan);
                start.elapsed()
            })
            .collect(),
    );
    assert_eq!(expected_pull_plan.counts.selected, 10_000);
    assert_eq!(expected_pull_plan.counts.actionable, 3_300);
    for directory_index in 0..100 {
        let source_directory = discovery_source.join(format!("directory-{directory_index:03}"));
        let destination_directory =
            discovery_destination.join(format!("directory-{directory_index:03}"));
        for file_index in 33..50 {
            std::fs::write(
                source_directory.join(format!("entry-{file_index:03}.txt")),
                b"source-only",
            )
            .unwrap();
        }
        for file_index in 50..66 {
            let name = format!("entry-{file_index:03}.txt");
            let source_file = source_directory.join(&name);
            let destination_file = destination_directory.join(name);
            std::fs::write(&source_file, b"converged").unwrap();
            std::fs::write(&destination_file, b"converged").unwrap();
            support::copy_complete_metadata(
                &source_file,
                &destination_file,
                grip::discovery::model::NodeKind::File,
            );
        }
        for file_index in 66..76 {
            let name = format!("entry-{file_index:03}.txt");
            std::fs::write(source_directory.join(&name), b"conflict-source").unwrap();
            std::fs::write(destination_directory.join(name), b"conflict-destination").unwrap();
        }
    }
    let dry_run_sync = measure_consistent(|| {
        let mut command = Command::new(&binary);
        command
            .env_clear()
            .env("HOME", &discovery_project)
            .current_dir(&discovery_project)
            .args(["--output=json", "sync", "--dry-run"]);
        command
    });
    let sync_state = grip::state::publication::load(&home).unwrap();
    let build_sync_plan = || {
        let observed =
            grip::observation::inspect(&home, &registry, &sync_state.accepted, &selection).unwrap();
        let records = observed
            .values()
            .map(|entry| grip::classification::classify_accepted(entry, &sync_state.accepted))
            .collect();
        grip::mutation::plan::build_sync_with_parent_requirements(
            grip::classification::model::ClassificationScope {
                kind: "all".into(),
                path_space: grip::observation::model::PathSpace::Source,
                selector: None,
                mapping_source: None,
            },
            records,
            registry.missing_destination_parents(),
        )
        .unwrap()
    };
    let expected_sync_plan = build_sync_plan();
    let sync_execute_plan = distribution(
        (0..sample_count())
            .map(|_| {
                let start = Instant::now();
                assert_eq!(build_sync_plan(), expected_sync_plan);
                start.elapsed()
            })
            .collect(),
    );
    assert_eq!(expected_sync_plan.counts.selected, 10_000);
    assert!(expected_sync_plan.counts.actionable > 0);
    assert!(expected_sync_plan.counts.converged > 0);
    assert!(expected_sync_plan.counts.blockers > 0);
    std::fs::remove_dir_all(&discovery_source).unwrap();
    std::fs::create_dir(&discovery_source).unwrap();
    let delete_preview = measure_consistent(|| {
        let mut command = Command::new(&binary);
        command
            .env_clear()
            .env("HOME", &discovery_project)
            .current_dir(&discovery_project)
            .args(["--output=json", "delete", "--dry-run", "--source"])
            .arg(&discovery_source);
        command
    });
    let recovery_home = root.path().join("recovery-home");
    std::fs::create_dir(&recovery_home).unwrap();
    support::initialize_project_metadata(&recovery_home);
    let operations = recovery_home.join(".grip/state/operations");
    std::fs::create_dir_all(&operations).unwrap();
    for index in 0..10_000 {
        std::fs::create_dir_all(operations.join(format!("operation-{index:05}/recovery"))).unwrap();
    }
    let recovery_inventory = measure_consistent(|| {
        let mut command = Command::new(&binary);
        command
            .env_clear()
            .env("HOME", root.path())
            .current_dir(&recovery_home)
            .args(["--output=json", "recovery", "list"]);
        command
    });
    let synchronization_container = root.path().join("synchronization-container");
    std::fs::create_dir(&synchronization_container).unwrap();
    let synchronization_home = support::initialize_project_metadata(&synchronization_container);
    let synchronization_source = synchronization_container.join("sources");
    let synchronization_destination = synchronization_container.join("destinations");
    std::fs::create_dir(&synchronization_source).unwrap();
    std::fs::create_dir(&synchronization_destination).unwrap();
    for directory_index in 0..100 {
        let source_directory =
            synchronization_source.join(format!("directory-{directory_index:03}"));
        let destination_directory =
            synchronization_destination.join(format!("directory-{directory_index:03}"));
        std::fs::create_dir(&source_directory).unwrap();
        std::fs::create_dir(&destination_directory).unwrap();
        for file_index in 0..99 {
            let source_file = source_directory.join(format!("entry-{file_index:03}"));
            let destination_file = destination_directory.join(format!("entry-{file_index:03}"));
            std::fs::write(&source_file, b"accepted").unwrap();
            std::fs::write(&destination_file, b"accepted").unwrap();
            let seconds = 1_710_000_000 + i64::from(directory_index * 100 + file_index);
            support::set_fixture_modified_time(&source_file, seconds, 246_813_579);
            support::set_fixture_modified_time(&destination_file, seconds, 246_813_579);
        }
        support::set_fixture_modified_time(&source_directory, 1_710_100_000, 135_792_468);
        support::set_fixture_modified_time(&destination_directory, 1_710_100_000, 135_792_468);
    }
    support::write_descriptor(
        &synchronization_home,
        &[(
            "tree",
            &synchronization_source,
            &synchronization_destination,
        )],
    );
    let initial_sync = Command::new(&binary)
        .env_clear()
        .env("HOME", &synchronization_container)
        .current_dir(&synchronization_container)
        .args(["baseline", "accept"])
        .output()
        .unwrap();
    assert!(initial_sync.status.success());
    for directory_index in 0..10 {
        std::fs::write(
            synchronization_source.join(format!("directory-{directory_index:03}/entry-000")),
            b"representative synchronization change",
        )
        .unwrap();
    }
    let synchronization_start = Instant::now();
    let synchronization = Command::new(&binary)
        .env_clear()
        .env("HOME", &synchronization_container)
        .current_dir(&synchronization_container)
        .arg("sync")
        .output()
        .unwrap();
    let synchronization_duration = synchronization_start.elapsed();
    assert!(synchronization.status.success());
    eprintln!(
        "distributions help={help:?} version={version:?} validate={validate:?} mapping_list_1000={mapping_list:?} implicit_discovery_100_levels={implicit_discovery_100_levels:?} discovery_10000={discovery:?} status_accepted_paired_10000={status:?} push_dry_run_10000={dry_run_push:?} push_execute_plan_10000={execute_mode_plan:?} pull_dry_run_10000={dry_run_pull:?} pull_execute_plan_10000={pull_execute_plan:?} sync_dry_run_mixed_10000={dry_run_sync:?} sync_execute_plan_mixed_10000={sync_execute_plan:?} delete_preview_10000={delete_preview:?} recovery_inventory_10000={recovery_inventory:?} representative_sync_10000_entries_10_changes={synchronization_duration:?}"
    );
    for measured in [
        help,
        version,
        validate,
        mapping_list,
        implicit_discovery_100_levels,
        discovery,
        status,
        dry_run_push,
        execute_mode_plan,
        dry_run_pull,
        pull_execute_plan,
        dry_run_sync,
        sync_execute_plan,
        delete_preview,
        recovery_inventory,
    ] {
        assert!(measured.p50 <= measured.p95 && measured.p95 <= measured.maximum);
    }
    assert!(help.p95 <= Duration::from_millis(100));
    assert!(version.p95 <= Duration::from_millis(100));
    assert!(validate.p95 <= Duration::from_secs(1));
    assert!(mapping_list.p95 <= Duration::from_secs(1));
    assert!(implicit_discovery_100_levels.p95 <= Duration::from_secs(1));
    assert!(discovery.p95 <= Duration::from_secs(2));
    assert!(status.p95 <= Duration::from_secs(2));
    assert!(dry_run_push.p95 <= Duration::from_secs(2));
    assert!(dry_run_pull.p95 <= Duration::from_secs(2));
    assert!(pull_execute_plan.p95 <= Duration::from_secs(2));
    assert!(dry_run_sync.p95 <= Duration::from_secs(2));
    assert!(sync_execute_plan.p95 <= Duration::from_secs(2));
    assert!(delete_preview.p95 <= Duration::from_secs(2));
    assert!(recovery_inventory.p95 <= Duration::from_secs(2));
}
