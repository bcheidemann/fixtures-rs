use std::path::Path;

use fixtures::fixtures;

#[fixtures([
    "fixtures/tests/fixtures/negative_globs/*.txt",
    "!fixtures/tests/fixtures/negative_globs/*.skip.txt",
])]
fn test(_path: &Path) {}

fn main() {}
