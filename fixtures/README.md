# fixtures

`fixtures` is a Rust crate which allows developers to run tests against fixture files. It provides a procedural macro
to generate tests from the filesystem, using glob patterns.

## Example

```rs
#[fixtures(["fixtures/*.txt"])]
#[test]
fn test(path: &std::path::Path) {
// This test will be run once for each file matching the glob pattern
}
```

To ensure tests re-run when the fixtures change, add the following line to `build.rs`.

```rs
fixtures::build::watch_dir("fixtures");
```

## Comparison to [`datatest-stable`](https://crates.io/crates/datatest-stable)

|                                                    | fixtures | datatest-stable |
| -------------------------------------------------- | -------- | --------------- |
| Requires setting `harness = false` in `Cargo.toml` | 🏅 no    | yes             |
| Supports non-test configurations e.g. `criterion`  | 🏅 yes   | no              |
| Supports embedding directories at compile time     | no       | 🏅 yes          |
| Works with `cargo-nextest`                         | 🏅 yes   | 🏅 yes          |
| Supports arbitrary function signatures             | 🏅 yes   | no              |
| Supports automatically injecting file contents     | no       | 🏅 yes          |
| Allows `#[ignore]`ing tests by glob patterns       | 🚧 wip   | no              |

## Usage

### Installation

```toml
[dependencies]
fixtures = "1"

[build-dependencies]
fixtures = "1"
```

### Setup

Add the following code to `build.rs` to watch your fixtures directories for changes.

```rs
// build.rs
use fixtures::build::watch_fixture_dir;

fn main() {
    watch_fixture_dir("path/to/fixtures");

    // or...

    watch_fixture_dir(&[
        "path/to/fixtures",
        // ...
    ]);
}
```

### Basic Usage

```rs
#[cfg(test)]
mod tests {
  use fixtures::fixtures;

  #[fixtures(["fixtures/*.txt"])]
  #[test]
  fn test(path: &std::path::Path) {
    // This test will be run once for each file matching the glob pattern
  }
}
```

The above example expands to:

```rs
#[cfg(test)]
mod tests {
  use fixtures::fixtures;

  fn test(path: &std::path::Path) {
    // This test will be run once for each file matching the glob pattern
  }

  #[test]
  fn test_one_dot_txt_1() {
    test(::std::path::Path::new("fixtures/one.txt"));
  }

  #[test]
  fn test_two_dot_txt_1() {
    test(::std::path::Path::new("fixtures/two.txt"));
  }

  // ...
}
```

### Multiple Globs

```rs
#[cfg(test)]
mod tests {
  use fixtures::fixtures;

  #[fixtures(["fixtures/*.txt", "fixtures/*.data"])]
  #[test]
  fn test(path: &std::path::Path) {
    // This test will be run once for each file matching either "fixtures/*.txt" or "fixtures/*.data"
  }
}
```

### Extended Glob Syntax

`fixtures` supports [`gitignore`'s extended glob syntax](https://git-scm.com/docs/gitignore#_pattern_format).

```rs
#[cfg(test)]
mod tests {
  use fixtures::fixtures;

  #[fixtures(["fixtures/*.{txt,data}", "!fixtures/skip.*.{txt,data}"])]
  #[test]
  fn test(path: &std::path::Path) {
    // This test will be run once for each fixture with the extension `txt` or `data`, unless it is prefixed with `skip.`
  }
}
```

## Advanced Usage

### Criterion

`fixtures` can be used with [`criterion`](https://github.com/bheisler/criterion.rs) as shown in the following example:

```rs
#[fixtures(["fixtures/bench/*"])]
fn bench(path: &std::path::Path, c: &mut Criterion) {
  let test_name = fixture_path.file_name().unwrap().to_str().unwrap();
  c.bench_function(test_name, |b| b.iter(|| { /* ... */ }));
}

// Equivalent to criterion_group!(benches, bench::expansion_1, bench::expansion_2, ...);
fn benches() {
  let mut c = Criterion::default().configure_from_args();
  for bench in bench::EXPANSIONS {
    bench(&mut criterion);
  }
}

criterion_main!(benches);
```
