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

## Comparison to [`datatest`](https://github.com/commure/datatest) and [`datatest-stable`](https://crates.io/crates/datatest-stable)

|                                                    | fixtures | datatest | datatest-stable |
| -------------------------------------------------- | -------- | -------- | --------------- |
| Supports stable rust                               | 🏅 yes   | no       | 🏅 yes          |
| Requires setting `harness = false` in `Cargo.toml` | 🏅 no    | no       | yes             |
| Supports non-test configurations e.g. `criterion`  | 🏅 yes   | no       | no              |
| Supports embedding directories at compile time     | no       | no       | 🏅 yes          |
| Works with `cargo-nextest`                         | 🏅 yes   | no       | 🏅 yes          |
| Supports arbitrary function signatures             | 🏅 yes   | no       | no              |
| Supports automatically injecting file contents     | no       | 🏅 yes   | 🏅 yes          |
| Allows `#[ignore]`ing tests by glob patterns       | 🏅 yes   | 🏅 yes   | no              |

## Usage

### Installation

```toml
[dependencies]
fixtures = "2"

[build-dependencies]
fixtures = "2"
```

### Setup

Add the following code to `build.rs` to watch your fixtures directories for changes.

```rs
fn main() {
  fixtures::build::watch_dir("path/to/fixtures");

  // or...

  fixtures::build::watch_dirs(&[
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

### Ignoring Files

Sometimes, you might want to ignore tests for one or more fixture files. If you want to skip generating the test
entirely, you can simply use a negative glob, as discussed above. However, if you instead want to `#[ignore]` the test,
you can do so as follows:

```rs
#[cfg(test)]
mod tests {
  use fixtures::fixtures;

  #[fixtures(
    ["fixtures/*.{txt,data}"],
    ignore = ["fixtures/ignored.txt"],
  )]
  #[test]
  fn test(path: &std::path::Path) {
    // This test will be run once for each fixture with the extension `txt` or `data`, unless it is prefixed with
    // `ignore.`, in which case the test will be decorated with `#[ignore = "Some reason for ignoring the test"]`
  }
}
```

In some cases you may wish to provide a reason for ignoring the test case.

```rs
#[cfg(test)]
mod tests {
  use fixtures::fixtures;

  #[fixtures(
    ["fixtures/*.{txt,data}"],
    ignore = [
      { path = "fixtures/ignored.txt", reason = "reason for ignoring file" }
    ],
  )]
  #[test]
  fn test(path: &std::path::Path) {}
}
```

The structure of the ignore option in EBNF notation is as follows:

```ebnf
Ignore                = IgnorePath | IgnorePathList | IgnoreObject ;

IgnorePathList        = "[]" | "[" IgnorePath { "," IgnorePath } [ "," ] "]" ;
IgnoreObject          = IgnorePathsOnly | IgnorePathsWithReason ;

IgnorePathsOnly       = "{" "paths" "=" (IgnorePath | IgnorePathList) [ "," ] "}" ;
IgnorePathsWithReason = "{" "paths" "=" (IgnorePath | IgnorePathList) "," "reason" "=" StringLiteral [ "," ] "}" ;

IgnorePath            = StringLiteral | IgnorePathObject ;
IgnorePathObject      = PathOnly | PathWithReason ;

PathOnly              = "{" "path" "=" StringLiteral [ "," ] "}" ;
PathWithReason        = "{" "path" "=" StringLiteral "," "reason" "=" StringLiteral [ "," ] "}" ;

StringLiteral         = /* Rust string literal */
```

This feature is only available for test functions; those with a `#[test]` attribute.

Note that the `ignore` glob will not be used to include files. This means that, for example, the ignore glob shown below
would have no effect, since none of the files matched by the include glob, are matched by the ignore glob.

```rs
#[fixtures(
  ["*.txt"],
  ignore = ["*.txt.ignore"], // This won't work!
)]
fn test(path: &std::path::Path) {}
```

This can be fixed as shown in the following example.

```rs
#[fixtures(
  ["*.txt{,.ignore}"],
  ignore = ["*.txt.ignore"], // This works as expected 🥳
)]
fn test(path: &std::path::Path) {}
```

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
