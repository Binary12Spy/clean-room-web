//! Build orchestration for the two-target workspace.
//!
//! A single `cargo build` cannot build both the native host and the
//! `wasm32-unknown-unknown` bundles. xtask drives them in the right order:
//! bundles first (so their `.wasm` artifacts exist), then the host.
//!
//! Usage:
//!   cargo xtask build            # build all bundles + host (debug)
//!   cargo xtask build --release  # release profile
//!   cargo xtask run [args...]    # build, then run the host with args
//!   cargo xtask check-oblivious  # assert the host names no layout library

use std::path::{Path, PathBuf};
use std::process::{Command, exit};

/// Bundle crates to compile for wasm32-unknown-unknown. Add new bundles here.
const BUNDLES: &[&str] = &["hello-rect", "app-todo-flex", "app-todo-grid"];

const WASM_TARGET: &str = "wasm32-unknown-unknown";

fn main() {
    let mut args = std::env::args().skip(1);
    let cmd = args.next().unwrap_or_else(|| "build".to_string());
    let rest: Vec<String> = args.collect();

    match cmd.as_str() {
        "build" => {
            let release = rest.iter().any(|a| a == "--release");
            build_all(release);
            println!(
                "\nbuilt. wasm artifacts in {}",
                wasm_out_dir(release).display()
            );
        }
        "run" => {
            // Allow `cargo xtask run --release -- <bundle> ...` or just args.
            let release = rest.iter().any(|a| a == "--release");
            build_all(release);
            run_host(release, &rest);
        }
        "check-oblivious" => check_oblivious(),
        other => {
            eprintln!("unknown command: {other}");
            eprintln!("commands: build [--release] | run [args] | check-oblivious");
            exit(2);
        }
    }
}

fn workspace_root() -> PathBuf {
    // CARGO_MANIFEST_DIR points at xtask/; parent is the workspace root.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask has a parent dir")
        .to_path_buf()
}

fn wasm_out_dir(release: bool) -> PathBuf {
    workspace_root()
        .join("target")
        .join(WASM_TARGET)
        .join(if release { "release" } else { "debug" })
}

fn build_all(release: bool) {
    for bundle in BUNDLES {
        build_bundle(bundle, release);
    }
    build_host(release);
}

fn build_bundle(name: &str, release: bool) {
    let dir = workspace_root().join("bundles").join(name);
    println!("==> building bundle `{name}` for {WASM_TARGET}");
    // Bundles are standalone crates (their own [workspace]); redirect their
    // artifacts into the shared workspace target dir so paths are predictable.
    let target_dir = workspace_root().join("target");
    let mut cmd = Command::new("cargo");
    cmd.current_dir(&dir)
        .arg("build")
        .arg("--target")
        .arg(WASM_TARGET)
        .arg("--target-dir")
        .arg(&target_dir);
    if release {
        cmd.arg("--release");
    }
    run(cmd, &format!("building bundle {name}"));
}

fn build_host(release: bool) {
    println!("==> building host (native)");
    let mut cmd = Command::new("cargo");
    cmd.current_dir(workspace_root())
        .arg("build")
        .arg("-p")
        .arg("host");
    if release {
        cmd.arg("--release");
    }
    run(cmd, "building host");
}

fn run_host(release: bool, rest: &[String]) {
    // Strip xtask-level flags; pass the remainder to the host binary.
    let host_args: Vec<&String> = rest.iter().filter(|a| a.as_str() != "--release").collect();
    let bin = workspace_root()
        .join("target")
        .join(if release { "release" } else { "debug" })
        .join("host");
    println!("==> running {}", bin.display());
    let mut cmd = Command::new(&bin);
    cmd.args(host_args);
    run(cmd, "running host");
}

/// The M3 keystone check: the host source must not name any layout library.
fn check_oblivious() {
    let host_src = workspace_root().join("host").join("src");
    let forbidden = ["flex", "grid", "Flex", "Grid"];
    let mut violations = Vec::new();

    visit_rs(&host_src, &mut |path, contents| {
        for needle in &forbidden {
            if contents.contains(needle) {
                violations.push(format!("{}: contains \"{needle}\"", path.display()));
            }
        }
    });

    if violations.is_empty() {
        println!("oblivious host check PASSED: no layout-library names in host/src");
    } else {
        eprintln!("oblivious host check FAILED:");
        for v in &violations {
            eprintln!("  {v}");
        }
        exit(1);
    }
}

fn visit_rs(dir: &Path, f: &mut impl FnMut(&Path, &str)) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            visit_rs(&path, f);
        } else if path.extension().is_some_and(|e| e == "rs")
            && let Ok(contents) = std::fs::read_to_string(&path)
        {
            f(&path, &contents);
        }
    }
}

fn run(mut cmd: Command, what: &str) {
    let status = cmd
        .status()
        .unwrap_or_else(|e| panic!("failed to spawn cargo while {what}: {e}"));
    if !status.success() {
        eprintln!("error: {what} failed ({status})");
        exit(status.code().unwrap_or(1));
    }
}
