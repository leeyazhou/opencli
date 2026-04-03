use assert_cmd::Command;

#[test]
fn config_help_smoke() {
    let mut cmd = Command::cargo_bin("ai-cli").unwrap();
    cmd.arg("config").arg("--help");
    cmd.assert().success();
}

#[test]
fn audit_help_smoke() {
    let mut cmd = Command::cargo_bin("ai-cli").unwrap();
    cmd.arg("audit").arg("--help");
    cmd.assert().success();
}

#[test]
fn completions_smoke() {
    let mut cmd = Command::cargo_bin("ai-cli").unwrap();
    cmd.arg("completions").arg("bash");
    cmd.assert().success();
}

#[test]
fn a2a_help_smoke() {
    let mut cmd = Command::cargo_bin("ai-cli").unwrap();
    cmd.arg("a2a").arg("--help");
    cmd.assert().success();
}

#[test]
fn a2a_batch_help_smoke() {
    let mut cmd = Command::cargo_bin("ai-cli").unwrap();
    cmd.arg("a2a-batch").arg("--help");
    cmd.assert().success();
}

#[test]
fn audit_graph_help_smoke() {
    let mut cmd = Command::cargo_bin("ai-cli").unwrap();
    cmd.arg("audit").arg("graph").arg("--help");
    cmd.assert().success();
}
