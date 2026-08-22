use std::process::Command;

// The hash is resolved here, at build time, so the binary carries it as a
// literal and never shells out to git to answer --version. A build outside a
// git checkout (a source tarball, a vendored copy) still has to produce
// something, hence the fallback.
fn main() {
    let hash = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|hash| !hash.is_empty())
        .unwrap_or_else(|| "unknown".to_string());

    println!("cargo:rustc-env=GIT_HASH={hash}");
    // Cargo caches this script's output, so name what invalidates it: a new
    // commit or a branch switch moves HEAD, and packed-refs covers the case
    // where the loose ref files are not there to watch.
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/heads");
    println!("cargo:rerun-if-changed=.git/packed-refs");
}
