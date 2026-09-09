mod support;

use grip::project::init::{InitializationFault, initialize_with_fault};
use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::sync::{Arc, Barrier};
use support::project::{PROJECT_GITIGNORE, ProjectFixture};

#[test]
fn init_rejects_missing_file_symlink_unsafe_and_nested_targets() {
    let fixture = ProjectFixture::new();
    let missing = fixture.root.path().join("missing");
    assert!(
        !fixture
            .command_from(fixture.root.path(), &["init", missing.to_str().unwrap()])
            .status
            .success()
    );

    let file = fixture.root.path().join("file");
    fs::write(&file, "x").unwrap();
    assert!(
        !fixture
            .command_from(fixture.root.path(), &["init", file.to_str().unwrap()])
            .status
            .success()
    );

    let link = fixture.root.path().join("link");
    symlink(&fixture.project_root, &link).unwrap();
    assert!(
        !fixture
            .command_from(fixture.root.path(), &["init", link.to_str().unwrap()])
            .status
            .success()
    );

    fs::set_permissions(&fixture.project_root, fs::Permissions::from_mode(0o777)).unwrap();
    assert!(!fixture.command(&["init"]).status.success());
    fs::set_permissions(&fixture.project_root, fs::Permissions::from_mode(0o755)).unwrap();

    fixture.write_descriptor(&[]);
    let nested = fixture.project_root.join("nested");
    fs::create_dir(&nested).unwrap();
    assert!(!fixture.command_from(&nested, &["init"]).status.success());
}

#[test]
fn init_rejects_partial_unsupported_and_noncanonical_metadata_without_repair() {
    let fixture = ProjectFixture::new();
    fs::create_dir(fixture.metadata_dir()).unwrap();
    fs::write(
        fixture.descriptor_path(),
        "schema_version = 1\nmappings = []\n",
    )
    .unwrap();
    fs::write(fixture.metadata_dir().join(".gitignore"), PROJECT_GITIGNORE).unwrap();
    let before = fixture.snapshot_all();
    assert!(!fixture.command(&["init"]).status.success());
    assert_eq!(fixture.snapshot_all(), before);

    fs::write(
        fixture.descriptor_path(),
        "schema_version = 2\nmappings = []\n",
    )
    .unwrap();
    fs::write(
        fixture.metadata_dir().join(".gitignore"),
        "/state/\nextra\n",
    )
    .unwrap();
    let before = fixture.snapshot_all();
    assert!(!fixture.command(&["init"]).status.success());
    assert_eq!(fixture.snapshot_all(), before);
}

#[test]
fn injected_failure_cleans_only_its_staging_and_publishes_nothing() {
    let fixture = ProjectFixture::new();
    assert!(
        initialize_with_fault(
            Some(&fixture.project_root),
            Some(InitializationFault::BeforePublish)
        )
        .is_err()
    );
    assert!(!fixture.metadata_dir().exists());
    assert!(fs::read_dir(&fixture.project_root).unwrap().all(|entry| {
        !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".grip-init-")
    }));
}

#[test]
fn concurrent_equivalent_initializers_produce_one_complete_project() {
    let fixture = ProjectFixture::new();
    let root = fixture.project_root.clone();
    let barrier = Arc::new(Barrier::new(3));
    let handles = (0..2)
        .map(|_| {
            let root = root.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                barrier.wait();
                grip::project::init::initialize(Some(&root))
                    .unwrap()
                    .outcome
            })
        })
        .collect::<Vec<_>>();
    barrier.wait();
    let outcomes = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect::<Vec<_>>();
    assert!(outcomes.contains(&grip::project::init::InitializationOutcome::Initialized));
    assert!(outcomes.contains(&grip::project::init::InitializationOutcome::AlreadyInitialized));
    assert_eq!(
        fs::read(fixture.metadata_dir().join(".gitignore")).unwrap(),
        b"/state/\n"
    );
}
