use std::path::Path;

pub fn watch_dir<P: AsRef<Path>>(path: P) {
    println!(
        "cargo:rerun-if-changed={}",
        path.as_ref().to_str().expect("path should be valid UTF-8")
    );
}

pub fn watch_dirs<P: AsRef<Path>>(paths: &[P]) {
    for path in paths {
        watch_dir(path);
    }
}

#[deprecated(since = "2.1.2", note = "Use `fixtures::build::watch_dir` instead.")]
pub fn watch_fixture_dir<P: AsRef<Path>>(path: P) {
    watch_dir(path);
}

#[deprecated(since = "2.1.2", note = "Use `fixtures::build::watch_dirs` instead.")]
pub fn watch_fixture_dirs<P: AsRef<Path>>(paths: &[P]) {
    watch_dirs(paths);
}
