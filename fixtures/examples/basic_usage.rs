use std::path::Path;

use fixtures::fixtures;

#[fixtures(["fixtures/tests/fixtures/basic_usage/*.txt"])]
fn test(_path: &Path) {}

fn main() {}
