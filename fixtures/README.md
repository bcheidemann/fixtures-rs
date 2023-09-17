# fixtures

`fixtures` is a Rust crate which allows developers to run tests against fixture files.

```rs
#[cfg(test)]
mod tests {
  use fixtures::fixtures;

  #[fixtures("fixtures/*.txt")]
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
