use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn lookup_rust_hashmap_iteration() {
    let mut command = Command::cargo_bin("codelex").unwrap();
    command.env("CODELEX_DATA_DIR", "/tmp/codelex-test-lookup");
    command
        .args(["rs", "hashmap", "iterate"])
        .assert()
        .success()
        .stdout(predicate::str::contains("for (key, value) in &map"));
}

#[test]
fn compare_hashmap_insert() {
    let mut command = Command::cargo_bin("codelex").unwrap();
    command.env("CODELEX_DATA_DIR", "/tmp/codelex-test-compare");
    command
        .args(["compare", "hashmap", "insert"])
        .assert()
        .success()
        .stdout(predicate::str::contains("map.insert(key, value);"));
}
