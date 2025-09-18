use std::path::Path;

use fixtures::fixtures;

#[fixtures(["fixtures/tests/fixtures/basic_usage/*.txt"])]
#[test]
fn test(_path: &Path) {}

fn main() {}
