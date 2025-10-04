#[cfg(test)]
use std::path::Path;

use fixtures::fixtures;

#[fixtures(["fixtures/tests/fixtures/ignore_globs/*.txt"])]
#[cfg_attr(
    test,
    fixtures::ignore(
        path = "fixtures/tests/fixtures/ignore_globs/*.ignore.txt",
        reason = "reason for ignoring file",
    )
)]
#[test]
fn test1(_path: &Path) {}

#[fixtures(["fixtures/tests/fixtures/ignore_globs/*.txt"])]
#[fixtures::ignore("fixtures/tests/fixtures/ignore_globs/*.ignore.txt")]
#[test]
fn test2(_path: &Path) {}

fn main() {}
