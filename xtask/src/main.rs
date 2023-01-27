use std::fs::create_dir_all;
use xtaskops::ops::{clean_files, cmd};

fn main() -> Result<(), anyhow::Error> {
    match std::env::args().nth(1).as_deref() {
        Some("cover") => cover(),
        _ => Ok(()),
    }
}

// https://blog.rng0.io/how-to-do-code-coverage-in-rust
fn cover() -> Result<(), anyhow::Error> {
    create_dir_all("coverage")?;

    println!("=== running coverage ===");
    cmd!("cargo", "test", "--package", "succinct_neo")
        .env("CARGO_INCREMENTAL", "0")
        .env("RUSTFLAGS", "-Cinstrument-coverage")
        .env("LLVM_PROFILE_FILE", "cargo-test-%p-%m.profraw")
        .run()?;
    println!("ok.");

    println!("=== generating report ===");
    let (fmt, file) = ("cobertura", "coverage/tests.xml");
    cmd!(
        "grcov",
        ".",
        "--binary-path",
        "./target/debug/deps",
        "-s",
        ".",
        "-t",
        fmt,
        "--branch",
        "--ignore-not-existing",
        "--ignore",
        "../*",
        "--ignore",
        "/*",
        "--ignore",
        "xtask/*",
        "--ignore",
        "*/src/tests/*",
        "-o",
        file,
    )
    .run()?;
    println!("ok.");

    println!("=== cleaning up ===");
    clean_files("**/*.profraw")?;
    println!("ok.");

    Ok(())
}
