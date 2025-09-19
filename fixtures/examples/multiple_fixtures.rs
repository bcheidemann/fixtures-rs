#[cfg(test)]
use std::path::Path;

use fixtures::fixtures;

#[fixtures(["fixtures/tests/fixtures/multiple_fixtures/*.txt"])]
#[test]
fn test(_path: &Path) {}

fn main() {}
