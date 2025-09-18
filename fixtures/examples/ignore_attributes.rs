use std::path::Path;

use fixtures::fixtures;

#[fixtures(["fixtures/tests/fixtures/ignore_attributes/*.txt"])]
#[ignore]
fn test(_path: &Path) {}

fn main() {}
