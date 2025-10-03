use std::path::Path;

use fixtures::fixtures;

#[fixtures(["fixtures/tests/fixtures/invalid_identifiers/*"])]
fn test(_path: &Path) {}

fn main() {}
