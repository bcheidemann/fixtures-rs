use indoc::indoc;
use std::process::Command;

fn test_example_expansion(example_name: &str) {
    let output = Command::new("cargo")
        .args(["expand", "--tests", "--example", example_name])
        .output()
        .expect("failed to expand example");

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!(
            indoc! {"
                error: failed to expand example ({})

                -- begin stdout --
                {}
                -- end stdout --

                -- begin stderr --
                {}
                -- end stderr --
            "},
            example_name, stdout, stderr
        );
    }

    let current_dir = std::env::current_dir()
        .expect("failed to get current directory")
        .to_string_lossy()
        .to_string();
    let expansion = String::from_utf8_lossy(&output.stdout).replace(&current_dir, "<repo>");

    insta::assert_snapshot!(example_name, expansion);
}

#[test]
fn basic_usage() {
    test_example_expansion("basic_usage");
}

#[test]
fn complex_signatures() {
    test_example_expansion("complex_signatures");
}

#[test]
fn criterion() {
    test_example_expansion("criterion");
}

#[test]
fn ignore_attributes() {
    test_example_expansion("ignore_attributes");
}

#[test]
fn multiple_fixtures() {
    test_example_expansion("multiple_fixtures");
}

#[test]
fn negative_globs() {
    test_example_expansion("negative_globs");
}
