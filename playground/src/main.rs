fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use fixtures::fixtures;
    use std::path::Path;

    #[fixtures("fixtures/**/*.txt")]
    fn hellooo(path: &Path) {
        let file = std::fs::File::open(path).unwrap();
        let data = std::io::read_to_string(file).unwrap();
        assert_eq!(data, path.file_name().unwrap().to_string_lossy());
    }
}
