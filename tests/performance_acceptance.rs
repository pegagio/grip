mod support;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{Duration, Instant};

const LARGE_FILE_BYTES: usize = 19 * 1024 * 1024;
const LARGE_FILE_SAMPLE_COUNT: usize = 100;
const LARGE_FILE_CHUNK_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, Copy)]
struct Distribution {
    p50: Duration,
    p95: Duration,
    maximum: Duration,
}

struct LargeFileFixture {
    root: tempfile::TempDir,
    project: PathBuf,
    source: PathBuf,
    destination: PathBuf,
}

impl LargeFileFixture {
    fn new() -> Self {
        let root = tempfile::tempdir_in("/private/tmp")
            .expect("could not create an isolated temporary root for performance acceptance");
        let project = root.path().join("project");
        fs::create_dir(&project)
            .expect("could not create an isolated project for performance acceptance");
        let _metadata = support::initialize_project_metadata(&project);
        let source = project.join("large-source.bin");
        let destination = project.join("large-destination.bin");
        write_large_file(&source, b'S');
        write_large_file(&destination, b'D');
        Self {
            root,
            project,
            source,
            destination,
        }
    }

    fn command(&self, binary: &Path, arguments: &[&str]) -> Command {
        let mut command = Command::new(binary);
        command
            .env_clear()
            .env("HOME", &self.project)
            .current_dir(&self.project)
            .args(arguments);
        command
    }

    fn assert_unequal_payloads(&self) {
        assert_eq!(
            fs::metadata(&self.source).unwrap().len(),
            LARGE_FILE_BYTES as u64
        );
        assert_eq!(
            fs::metadata(&self.destination).unwrap().len(),
            LARGE_FILE_BYTES as u64
        );
        assert_ne!(
            fs::read(&self.source).unwrap(),
            fs::read(&self.destination).unwrap()
        );
    }

    fn root_path(&self) -> &Path {
        self.root.path()
    }
}

fn write_large_file(path: &Path, byte: u8) {
    let chunk = vec![byte; LARGE_FILE_CHUNK_BYTES];
    let mut file = std::fs::File::create(path).unwrap_or_else(|error| {
        panic!(
            "could not create large-file fixture {}: {error}",
            path.display()
        )
    });
    for _ in 0..(LARGE_FILE_BYTES / LARGE_FILE_CHUNK_BYTES) {
        std::io::Write::write_all(&mut file, &chunk).unwrap_or_else(|error| {
            panic!(
                "could not write large-file fixture {}: {error}",
                path.display()
            )
        });
    }
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

fn assert_deterministic_output(label: &str, expected: &Output, actual: &Output) {
    assert_eq!(
        actual.status.code(),
        expected.status.code(),
        "{label}: unexpected exit status"
    );
    assert_eq!(actual.stdout, expected.stdout, "{label}: unexpected stdout");
    assert_eq!(actual.stderr, expected.stderr, "{label}: unexpected stderr");
}

fn measure_outputs(label: &str, runs: usize, mut command: impl FnMut() -> Output) -> Distribution {
    let expected = command();
    assert!(
        expected.status.code().is_some(),
        "{label}: command did not produce an exit status"
    );
    distribution(
        (0..runs)
            .map(|_| {
                let start = Instant::now();
                let output = command();
                assert_deterministic_output(label, &expected, &output);
                start.elapsed()
            })
            .collect(),
    )
}

fn measure(label: &str, mut command: impl FnMut() -> Command) -> Distribution {
    measure_outputs(label, sample_count(), || command().output().unwrap())
}

fn release_binary() -> PathBuf {
    let binary = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/release/grip");
    assert!(
        binary.is_file(),
        "release binary is unavailable at {}; run cargo build --release or mise run performance",
        binary.display()
    );
    binary
}

fn run_large_add(binary: &Path) -> Output {
    let fixture = LargeFileFixture::new();
    let output = fixture
        .command(
            binary,
            &["add", "large-source.bin", "~/large-destination.bin"],
        )
        .output()
        .expect("could not run release grip add for the isolated large-file workload");
    assert!(
        output.status.success(),
        "large-file add failed: {}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    fixture.assert_unequal_payloads();
    let status = fixture
        .command(binary, &["--output=json", "status"])
        .output()
        .expect("could not verify isolated large-file add semantics");
    assert!(
        status.status.success(),
        "large-file add semantic verification failed: {}{}",
        String::from_utf8_lossy(&status.stdout),
        String::from_utf8_lossy(&status.stderr)
    );
    let status_json: serde_json::Value = serde_json::from_slice(&status.stdout).unwrap();
    assert_eq!(
        status_json["details"]["records"][0]["classification"],
        "source_only_change"
    );
    assert!(fixture.root_path().starts_with("/private/tmp"));
    output
}

fn large_status_fixture(binary: &Path) -> LargeFileFixture {
    let fixture = LargeFileFixture::new();
    let output = fixture
        .command(
            binary,
            &["add", "large-source.bin", "~/large-destination.bin"],
        )
        .output()
        .expect("could not establish the isolated large-file status workload");
    assert!(
        output.status.success(),
        "could not establish large-file status workload: {}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    fixture.assert_unequal_payloads();
    fixture
}

#[test]
#[ignore = "representative-workstation acceptance harness"]
fn warm_release_commands_meet_p95_targets() {
    let binary = release_binary();
    let root = tempfile::tempdir_in("/private/tmp")
        .expect("could not create an isolated temporary root for performance acceptance");
    let project = root.path().join("project");
    fs::create_dir(&project)
        .expect("could not create an isolated project for performance acceptance");
    let _metadata = support::initialize_project_metadata(&project);
    let source_root = project.join("sources");
    let destination_root = project.join("destinations");
    fs::create_dir(&source_root).unwrap();
    fs::create_dir(&destination_root).unwrap();
    for index in 0..1_000 {
        let source = source_root.join(format!("entry-{index:04}"));
        let destination = destination_root.join(format!("entry-{index:04}"));
        fs::write(&source, index.to_string()).unwrap();
        fs::write(&destination, index.to_string()).unwrap();
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

    let large_add = measure_outputs("large-file add", LARGE_FILE_SAMPLE_COUNT, || {
        run_large_add(&binary)
    });
    let status_fixture = large_status_fixture(&binary);
    let large_status = measure_outputs("large-file status", LARGE_FILE_SAMPLE_COUNT, || {
        status_fixture
            .command(&binary, &["--output=json", "status"])
            .output()
            .expect("could not run release grip status for the isolated large-file workload")
    });

    eprintln!(
        "runs={} large_file_bytes={} large_file_runs={} target_os={} target_arch={} binary={} help={help:?} version={version:?} list={list:?} status={status:?} diff={diff:?} push_dry_run={push_dry_run:?} pull_dry_run={pull_dry_run:?} sync_dry_run={sync_dry_run:?} large_add={large_add:?} large_status={large_status:?}",
        sample_count(),
        LARGE_FILE_BYTES,
        LARGE_FILE_SAMPLE_COUNT,
        std::env::consts::OS,
        std::env::consts::ARCH,
        binary.display(),
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
        large_add,
        large_status,
    ] {
        assert!(measured.p50 <= measured.p95 && measured.p95 <= measured.maximum);
    }
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
        assert!(measured.p95 <= Duration::from_secs(2));
    }
    for (label, measured) in [
        ("large-file add", large_add),
        ("large-file status", large_status),
    ] {
        assert!(
            measured.p95 <= Duration::from_secs(1),
            "{label} p95 exceeded the one-second target: {measured:?}"
        );
    }
    assert!(help.p95 <= Duration::from_millis(100));
    assert!(version.p95 <= Duration::from_millis(100));
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[test]
#[ignore = "contained-home representative-workstation acceptance harness"]
fn contained_home_status_is_bounded_by_managed_identity_count() {
    const STATUS_SAMPLES: usize = 100;
    let binary = release_binary();
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let home = root.path().join("home");
    let project = home.join("dotfiles");
    let source = project.join("home");
    fs::create_dir_all(&source).unwrap();
    let _metadata = support::initialize_project_metadata(&project);
    for index in 0..100 {
        fs::write(
            source.join(format!("managed-{index:03}")),
            index.to_string(),
        )
        .unwrap();
    }
    let unrelated = home.join("unrelated");
    fs::create_dir(&unrelated).unwrap();
    for index in 0..10_000 {
        fs::write(unrelated.join(format!("item-{index:05}")), b"").unwrap();
    }
    let link_target = root.path().join("unmanaged-link-target");
    fs::write(&link_target, b"Grip must not inspect this target").unwrap();
    std::os::unix::fs::symlink(&link_target, home.join("managed-000")).unwrap();
    let add = Command::new(&binary)
        .env_clear()
        .env("HOME", &home)
        .current_dir(&project)
        .args(["add", "home/", "~/"])
        .output()
        .unwrap();
    assert!(
        add.status.success(),
        "{}",
        String::from_utf8_lossy(&add.stderr)
    );
    assert!(
        home.join("managed-000")
            .symlink_metadata()
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert_eq!(
        fs::read(&link_target).unwrap(),
        b"Grip must not inspect this target"
    );

    let measured = measure_outputs("contained-home status", STATUS_SAMPLES, || {
        Command::new(&binary)
            .env_clear()
            .env("HOME", &home)
            .current_dir(&project)
            .args(["--output=json", "status"])
            .output()
            .unwrap()
    });
    assert!(
        measured.p95 <= Duration::from_secs(1),
        "contained-home status p95 exceeded one second: {measured:?}"
    );
}
