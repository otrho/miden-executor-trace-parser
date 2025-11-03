fn check(log_path: &str, entry_point: &str) {
    let full_log_path = "tests/".to_string() + log_path;
    let output = test_bin::get_test_bin!("etp")
        .args(["-e", entry_point, &full_log_path])
        .output()
        .expect("Failed to run `etp`");
    assert!(output.status.success());

    let expected_path = log_path.to_string() + ".expected";
    expect_test::expect_file!(expected_path).assert_eq(&String::from_utf8_lossy(&output.stdout));
}

#[test]
fn test_short() {
    check("short.log", "main");
}

#[test]
fn test_assert() {
    check("break_on_assert.log", "#run");
}
