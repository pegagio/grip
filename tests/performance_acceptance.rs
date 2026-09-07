mod support;
use std::process::Command;
use std::time::{Duration, Instant};

fn p95(mut samples: Vec<Duration>) -> Duration {
    samples.sort_unstable();
    let index = (samples.len() * 95).div_ceil(100).saturating_sub(1);
    samples[index]
}
fn sample_count() -> usize {
    std::env::var("GRIP_PERFORMANCE_SAMPLE_COUNT")
        .ok()
        .and_then(|value| value.parse().ok())
        .filter(|count| *count > 0)
        .unwrap_or(100)
}
fn measure(mut command: impl FnMut() -> Command) -> Duration {
    let _ = command().output().unwrap();
    let samples = (0..sample_count())
        .map(|_| {
            let start = Instant::now();
            let output = command().output().unwrap();
            assert!(output.status.success());
            start.elapsed()
        })
        .collect();
    p95(samples)
}

fn measure_consistent(mut command: impl FnMut() -> Command) -> Duration {
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
    p95(samples)
}

#[test]
#[ignore = "representative-workstation acceptance harness"]
fn warm_release_commands_meet_p95_targets() {
    let binary = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/release/grip");
    assert!(binary.is_file(), "build release binary first");
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let grip_home = support::minimal_home(root.path());
    let source_root = root.path().join("sources");
    let destination_root = root.path().join("destinations");
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
    support::write_registry(&grip_home, &borrowed);
    eprintln!(
        "os={} arch={} rust={} profile=release runs={}",
        std::env::consts::OS,
        std::env::consts::ARCH,
        env!("CARGO_PKG_RUST_VERSION"),
        sample_count()
    );
    let help = measure(|| {
        let mut c = Command::new(&binary);
        c.env_clear().env("HOME", root.path()).arg("--help");
        c
    });
    let version = measure(|| {
        let mut c = Command::new(&binary);
        c.env_clear().env("HOME", root.path()).arg("version");
        c
    });
    let validate = measure(|| {
        let mut c = Command::new(&binary);
        c.env_clear()
            .env("HOME", root.path())
            .env("GRIP_HOME", &grip_home)
            .arg("validate");
        c
    });
    let mapping_list = measure_consistent(|| {
        let mut c = Command::new(&binary);
        c.env_clear()
            .env("HOME", root.path())
            .env("GRIP_HOME", &grip_home)
            .args(["--output=json", "mapping", "list"]);
        c
    });
    let discovery_home = root.path().join("discovery-home");
    let discovery_source = root.path().join("discovery-source");
    let discovery_destination = root.path().join("discovery-destination");
    std::fs::create_dir(&discovery_home).unwrap();
    std::fs::create_dir(&discovery_source).unwrap();
    std::fs::create_dir(&discovery_destination).unwrap();
    for directory_index in 0..100 {
        let relative_directory = format!("directory-{directory_index:03}");
        let directory = discovery_source.join(&relative_directory);
        let destination_directory = discovery_destination.join(&relative_directory);
        std::fs::create_dir(&directory).unwrap();
        std::fs::create_dir(&destination_directory).unwrap();
        for file_index in 33..99 {
            let name = format!("entry-{file_index:03}.txt");
            std::fs::write(directory.join(&name), b"representative").unwrap();
            std::fs::write(destination_directory.join(name), b"representative").unwrap();
        }
    }
    std::fs::write(
        discovery_source.join("directory-050/.gripignore"),
        "*.ignored\n",
    )
    .unwrap();
    support::write_registry(
        &discovery_home,
        &[("tree", &discovery_source, &discovery_destination)],
    );
    let discovery = measure_consistent(|| {
        let mut command = Command::new(&binary);
        command
            .env_clear()
            .env("HOME", root.path())
            .env("GRIP_HOME", &discovery_home)
            .args(["--output=json", "mapping", "inspect"])
            .arg(&discovery_source);
        command
    });
    let accepted = Command::new(&binary)
        .env_clear()
        .env("HOME", root.path())
        .env("GRIP_HOME", &discovery_home)
        .args(["baseline", "accept"])
        .output()
        .unwrap();
    assert!(accepted.status.success());
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
            .env("HOME", root.path())
            .env("GRIP_HOME", &discovery_home)
            .args(["--output=json", "status"]);
        command
    });
    let dry_run_push = measure_consistent(|| {
        let mut command = Command::new(&binary);
        command
            .env_clear()
            .env("HOME", root.path())
            .env("GRIP_HOME", &discovery_home)
            .args(["--output=json", "push", "--dry-run"]);
        command
    });
    let home = grip::home::select(Some(discovery_home.clone().into_os_string()), None).unwrap();
    let registry = grip::registry::publication::load(&home, false).unwrap();
    let state = grip::state::publication::load(&home).unwrap();
    let selection = grip::observation::model::Selection::All;
    let build_plan = || {
        let observed =
            grip::observation::inspect(&home, &registry, &state.accepted, &selection).unwrap();
        let records = observed
            .values()
            .map(|entry| {
                grip::classification::classify(entry, state.accepted.baselines.get(&entry.identity))
            })
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
    let execute_mode_plan = p95((0..sample_count())
        .map(|_| {
            let start = Instant::now();
            assert_eq!(build_plan(), expected_plan);
            start.elapsed()
        })
        .collect());
    let dry_run_output = Command::new(&binary)
        .env_clear()
        .env("HOME", root.path())
        .env("GRIP_HOME", &discovery_home)
        .args(["--output=json", "push", "--dry-run"])
        .output()
        .unwrap();
    assert_eq!(
        support::json(&dry_run_output)["details"]["plan_id"]["digest"],
        expected_plan.plan_id
    );
    assert_eq!(expected_plan.counts.actionable, 6_600);

    for directory_index in 0..100 {
        let source_directory = discovery_source.join(format!("directory-{directory_index:03}"));
        let destination_directory =
            discovery_destination.join(format!("directory-{directory_index:03}"));
        for file_index in 0..66 {
            std::fs::copy(
                source_directory.join(format!("entry-{file_index:03}.txt")),
                destination_directory.join(format!("entry-{file_index:03}.txt")),
            )
            .unwrap();
        }
    }
    let pull_accepted = Command::new(&binary)
        .env_clear()
        .env("HOME", root.path())
        .env("GRIP_HOME", &discovery_home)
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
            .env("HOME", root.path())
            .env("GRIP_HOME", &discovery_home)
            .args(["--output=json", "pull", "--dry-run"]);
        command
    });
    let pull_state = grip::state::publication::load(&home).unwrap();
    let build_pull_plan = || {
        let observed =
            grip::observation::inspect(&home, &registry, &pull_state.accepted, &selection).unwrap();
        let records = observed
            .values()
            .map(|entry| {
                grip::classification::classify(
                    entry,
                    pull_state.accepted.baselines.get(&entry.identity),
                )
            })
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
    let pull_execute_plan = p95((0..sample_count())
        .map(|_| {
            let start = Instant::now();
            assert_eq!(build_pull_plan(), expected_pull_plan);
            start.elapsed()
        })
        .collect());
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
            std::fs::write(source_directory.join(&name), b"converged").unwrap();
            std::fs::write(destination_directory.join(name), b"converged").unwrap();
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
            .env("HOME", root.path())
            .env("GRIP_HOME", &discovery_home)
            .args(["--output=json", "sync", "--dry-run"]);
        command
    });
    let sync_state = grip::state::publication::load(&home).unwrap();
    let build_sync_plan = || {
        let observed =
            grip::observation::inspect(&home, &registry, &sync_state.accepted, &selection).unwrap();
        let records = observed
            .values()
            .map(|entry| {
                grip::classification::classify(
                    entry,
                    sync_state.accepted.baselines.get(&entry.identity),
                )
            })
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
    let sync_execute_plan = p95((0..sample_count())
        .map(|_| {
            let start = Instant::now();
            assert_eq!(build_sync_plan(), expected_sync_plan);
            start.elapsed()
        })
        .collect());
    assert_eq!(expected_sync_plan.counts.selected, 10_000);
    assert!(expected_sync_plan.counts.actionable > 0);
    assert!(expected_sync_plan.counts.converged > 0);
    assert!(expected_sync_plan.counts.blockers > 0);
    eprintln!(
        "p95 help={help:?} version={version:?} validate={validate:?} mapping_list_1000={mapping_list:?} discovery_10000={discovery:?} status_accepted_paired_10000={status:?} push_dry_run_10000={dry_run_push:?} push_execute_plan_10000={execute_mode_plan:?} pull_dry_run_10000={dry_run_pull:?} pull_execute_plan_10000={pull_execute_plan:?} sync_dry_run_mixed_10000={dry_run_sync:?} sync_execute_plan_mixed_10000={sync_execute_plan:?}"
    );
    assert!(help <= Duration::from_millis(100));
    assert!(version <= Duration::from_millis(100));
    assert!(validate <= Duration::from_secs(1));
    assert!(mapping_list <= Duration::from_secs(1));
    assert!(discovery <= Duration::from_secs(2));
    assert!(status <= Duration::from_secs(2));
    assert!(dry_run_push <= Duration::from_secs(2));
    assert!(dry_run_pull <= Duration::from_secs(2));
    assert!(pull_execute_plan <= Duration::from_secs(2));
    assert!(dry_run_sync <= Duration::from_secs(2));
    assert!(sync_execute_plan <= Duration::from_secs(2));
}
