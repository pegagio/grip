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
    let percentile =
        |value: usize| samples[(samples.len() * value).div_ceil(100).saturating_sub(1)];
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
        .unwrap_or(25)
}

fn measure(label: &str, mut command: impl FnMut() -> Command) -> Distribution {
    let expected = command().output().unwrap();
    assert!(
        expected.status.code().is_some(),
        "{label}: command did not produce an exit status"
    );
    distribution(
        (0..sample_count())
            .map(|_| {
                let start = Instant::now();
                let output = command().output().unwrap();
                assert_eq!(output.status.code(), expected.status.code());
                assert_eq!(output.stdout, expected.stdout);
                assert_eq!(output.stderr, expected.stderr);
                start.elapsed()
            })
            .collect(),
    )
}

#[test]
#[ignore = "representative-workstation acceptance harness"]
fn warm_release_commands_meet_p95_targets() {
    let binary = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/release/grip");
    assert!(binary.is_file(), "build release binary first");
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let project = root.path().join("project");
    std::fs::create_dir(&project).unwrap();
    let _metadata = support::initialize_project_metadata(&project);
    let source_root = project.join("sources");
    let destination_root = project.join("destinations");
    std::fs::create_dir(&source_root).unwrap();
    std::fs::create_dir(&destination_root).unwrap();
    for index in 0..1_000 {
        let source = source_root.join(format!("entry-{index:04}"));
        let destination = destination_root.join(format!("entry-{index:04}"));
        std::fs::write(&source, index.to_string()).unwrap();
        std::fs::write(&destination, index.to_string()).unwrap();
    }
    let added = Command::new(&binary)
        .env_clear()
        .env("HOME", &project)
        .current_dir(&project)
        .args(["add", "sources", "~/destinations"])
        .output()
        .unwrap();
    assert!(
        added.status.success(),
        "{}{}",
        String::from_utf8_lossy(&added.stdout),
        String::from_utf8_lossy(&added.stderr)
    );

    let command = |arguments: &'static [&'static str]| {
        let binary = binary.clone();
        let project = project.clone();
        move || {
            let mut command = Command::new(&binary);
            command
                .env_clear()
                .env("HOME", &project)
                .current_dir(&project)
                .args(arguments);
            command
        }
    };
    let help = measure("help", command(&["--help"]));
    let version = measure("version", command(&["version"]));
    let list = measure("list", command(&["--output=json", "list"]));
    let status = measure("status", command(&["--output=json", "status"]));
    let diff = measure("diff", command(&["--output=json", "diff"]));
    let push_dry_run = measure(
        "push dry-run",
        command(&["--output=json", "push", "--dry-run"]),
    );
    let pull_dry_run = measure(
        "pull dry-run",
        command(&["--output=json", "pull", "--dry-run"]),
    );
    let sync_dry_run = measure(
        "sync dry-run",
        command(&["--output=json", "sync", "--dry-run"]),
    );

    eprintln!(
        "runs={} help={help:?} version={version:?} list={list:?} status={status:?} diff={diff:?} push_dry_run={push_dry_run:?} pull_dry_run={pull_dry_run:?} sync_dry_run={sync_dry_run:?}",
        sample_count()
    );
    for measured in [
        help,
        version,
        list,
        status,
        diff,
        push_dry_run,
        pull_dry_run,
        sync_dry_run,
    ] {
        assert!(measured.p50 <= measured.p95 && measured.p95 <= measured.maximum);
        assert!(measured.p95 <= Duration::from_secs(2));
    }
    assert!(help.p95 <= Duration::from_millis(100));
    assert!(version.p95 <= Duration::from_millis(100));
}
