#[cfg(test)]
use std::path::Path;

use fixtures::fixtures;

#[fixtures(
    ["fixtures/tests/fixtures/legacy_ignore_globs/*.txt"],
    ignore = {
        paths = ["fixtures/tests/fixtures/legacy_ignore_globs/*.ignore.txt"],
        reason = "reason for ignoring file",
    },
)]
#[test]
fn test1(_path: &Path) {}

#[fixtures(
    ["fixtures/tests/fixtures/legacy_ignore_globs/*.txt"],
    ignore = ["fixtures/tests/fixtures/legacy_ignore_globs/*.ignore.txt"],
)]
#[test]
fn test2(_path: &Path) {}

fn main() {}
