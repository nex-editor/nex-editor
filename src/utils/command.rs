use std::process::Command;

// git commit id
pub fn get_commit_id() -> String {
    let output = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .expect("failed to execute process");
    let commit_id = String::from_utf8(output.stdout).unwrap();

    commit_id.chars().take(7).collect()
}

// git tag
pub fn get_tag() -> String {
    let output = Command::new("git")
        .args(&["describe", "--tags"])
        .output()
        .expect("failed to execute process");
    let tag = String::from_utf8(output.stdout).unwrap();
    tag.trim().to_string()
}
