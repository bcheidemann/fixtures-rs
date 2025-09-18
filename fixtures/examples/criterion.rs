use std::path::Path;

use fixtures::fixtures;

struct Criterion;

#[fixtures(["fixtures/tests/fixtures/criterion/*.txt"])]
fn bench(_path: &Path, _c: &mut Criterion) {}

fn benches() {
    let mut criterion = Criterion;

    for bench in bench::EXPANSIONS {
        bench(&mut criterion);
    }
}

fn main() {
    benches();
}
