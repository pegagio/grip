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

#[test]
#[ignore = "representative-workstation acceptance harness"]
fn warm_release_commands_meet_p95_targets() {
    let binary = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/release/grip");
    assert!(binary.is_file(), "build release binary first");
    let root = tempfile::tempdir().unwrap();
    let grip_home = support::minimal_home(root.path());
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
    eprintln!("p95 help={help:?} version={version:?} validate={validate:?}");
    assert!(help <= Duration::from_millis(100));
    assert!(version <= Duration::from_millis(100));
    assert!(validate <= Duration::from_millis(250));
}
