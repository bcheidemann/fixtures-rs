#[cfg(test)]
use std::path::Path;

use fixtures::fixtures;

#[fixtures(
    ["fixtures/tests/fixtures/complex_legacy_ignore_config/*.txt"],
    ignore = {
        paths = [
            "fixtures/tests/fixtures/complex_legacy_ignore_config/file_2.txt",
            {
                path = "fixtures/tests/fixtures/complex_legacy_ignore_config/file_3.txt",
                reason = "specific reason for ignoring file 3",
            }
        ],
        reason = "default reason for ignoring file",
    },
)]
#[test]
fn test(_path: &Path) {}

fn main() {}
