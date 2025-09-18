use std::path::Path;

use fixtures::fixtures;

#[fixtures(["fixtures/tests/fixtures/complex_signatures/*.txt"])]
fn complex_fn(
    _path: &Path,
    _owned: String,
    _mutable: &mut String,
    _borrowed: &str,
) -> Result<String, ()> {
    Ok("hello world!".to_string())
}

fn main() {
    complex_fn::file_1_dot_txt("owned".to_string(), &mut "mutable".to_string(), "borrowed")
        .unwrap();
}
