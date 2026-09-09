use grip::home;
use std::fs;

#[test]
fn selects_and_canonicalizes_an_explicit_invoking_user_home() {
    let root = tempfile::tempdir().unwrap();
    assert_eq!(
        home::select_user_home(Some(root.path().to_owned()))
            .unwrap()
            .path(),
        fs::canonicalize(root.path()).unwrap()
    );
    assert!(home::select_user_home(Some("relative".into())).is_err());
}

#[cfg(unix)]
#[test]
fn rejects_symlink_and_wrong_node_type() {
    use std::os::unix::fs::symlink;
    let root = tempfile::tempdir().unwrap();
    let directory = root.path().join("directory");
    fs::create_dir(&directory).unwrap();
    let link = root.path().join("link");
    symlink(&directory, &link).unwrap();
    assert!(home::select_user_home(Some(link)).is_err());
    let file = root.path().join("file");
    fs::write(&file, "x").unwrap();
    assert!(home::select_user_home(Some(file)).is_err());
}
