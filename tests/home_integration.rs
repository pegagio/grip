use grip::home::{self, HomeAccess};
use std::cell::RefCell;
use std::fs::{self, File, Metadata};
use std::path::{Path, PathBuf};

struct RecordingAccess {
    paths: RefCell<Vec<PathBuf>>,
}
impl HomeAccess for RecordingAccess {
    fn symlink_metadata(&self, p: &Path) -> std::io::Result<Metadata> {
        self.paths.borrow_mut().push(p.to_owned());
        fs::symlink_metadata(p)
    }
    fn open_dir(&self, p: &Path) -> std::io::Result<File> {
        self.paths.borrow_mut().push(p.to_owned());
        File::open(p)
    }
}

#[test]
fn selects_default_or_exact_absolute_override() {
    let root = tempfile::tempdir().unwrap();
    assert_eq!(
        home::select(None, Some(root.path().to_owned()))
            .unwrap()
            .path(),
        root.path().join(".grip")
    );
    let exact = root.path().join("custom");
    assert_eq!(
        home::select(Some(exact.clone().into_os_string()), None)
            .unwrap()
            .path(),
        exact
    );
    assert!(home::select(Some("relative".into()), None).is_err());
    assert!(home::select(Some("".into()), None).is_err());
}

#[test]
fn validation_accesses_only_selected_root() {
    let root = tempfile::tempdir().unwrap();
    let selected = root.path().join("selected");
    fs::create_dir(&selected).unwrap();
    let home = home::select(Some(selected.clone().into_os_string()), None).unwrap();
    let access = RecordingAccess {
        paths: RefCell::new(Vec::new()),
    };
    home::validate_with(&access, &home).unwrap();
    assert!(access.paths.borrow().iter().all(|p| p == &selected));
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
    assert!(home::validate(&home::select(Some(link.into_os_string()), None).unwrap()).is_err());
    let file = root.path().join("file");
    fs::write(&file, "x").unwrap();
    assert!(home::validate(&home::select(Some(file.into_os_string()), None).unwrap()).is_err());
}
