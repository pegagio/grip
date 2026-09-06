mod support;
use std::process::Command;
use std::time::{Duration, Instant};

fn p95(mut samples: Vec<Duration>) -> Duration {
    samples.sort_unstable();
    samples[94]
}
fn measure(mut command: impl FnMut() -> Command) -> Duration {
    let _ = command().output().unwrap();
    let samples = (0..100)
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
    assert!(expected.status.success());
    let samples = (0..100)
        .map(|_| {
            let start = Instant::now();
            let output = command().output().unwrap();
            let elapsed = start.elapsed();
            assert!(output.status.success());
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
    let root = tempfile::tempdir().unwrap();
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
        "os={} arch={} rust={} profile=release runs=100",
        std::env::consts::OS,
        std::env::consts::ARCH,
        env!("CARGO_PKG_RUST_VERSION")
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
        for file_index in 0..99 {
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
    let status = measure_consistent(|| {
        let mut command = Command::new(&binary);
        command
            .env_clear()
            .env("HOME", root.path())
            .env("GRIP_HOME", &discovery_home)
            .args(["--output=json", "status"]);
        command
    });
    eprintln!(
        "p95 help={help:?} version={version:?} validate={validate:?} mapping_list_1000={mapping_list:?} discovery_10000={discovery:?} status_accepted_paired_10000={status:?}"
    );
    assert!(help <= Duration::from_millis(100));
    assert!(version <= Duration::from_millis(100));
    assert!(validate <= Duration::from_secs(1));
    assert!(mapping_list <= Duration::from_secs(1));
    assert!(discovery <= Duration::from_secs(2));
    assert!(status <= Duration::from_secs(2));
}
