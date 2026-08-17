fn main() {
    // `git rev-parse` outside a repository exits non-zero with empty stdout, and
    // that is an Ok(Output) as far as `Command` is concerned — so the status has
    // to be checked explicitly or the fallback below never runs and `--version`
    // reports an empty SHA. T26 builds from source archives that carry no .git.
    let sha = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|sha| !sha.is_empty())
        .unwrap_or_else(|| "unknown".to_string());

    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown-unknown-unknown".to_string());

    // Without this the SHA is baked in until some source file changes, so a
    // commit that touches nothing else leaves `--version` reporting the old one.
    println!("cargo:rerun-if-changed=../../.git/HEAD");
    println!("cargo:rustc-env=CARGO_TRESTLE_GIT_SHA={}", sha);
    println!("cargo:rustc-env=CARGO_TRESTLE_TARGET_TRIPLE={}", target);
}
