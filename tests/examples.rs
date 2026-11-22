//! Runs every program in `examples/` and compares what it prints with the
//! `.out` file beside it. A `.in` file, if present, is fed to the program as
//! its input. Programs named `*-endless.mouse` never stop and are skipped.

use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

#[test]
fn examples_print_their_expected_output() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    let mut programs: Vec<_> = fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "mouse"))
        .collect();
    programs.sort();

    let mut checked = 0;
    let mut failures = Vec::new();
    for program in &programs {
        let name = program.file_stem().unwrap().to_string_lossy().into_owned();
        if name.ends_with("-endless") {
            continue;
        }
        let expected = fs::read_to_string(program.with_extension("out"))
            .unwrap_or_else(|_| panic!("{name}.mouse has no {name}.out"));
        let input = fs::read(program.with_extension("in")).unwrap_or_default();

        let mut child = Command::new(env!("CARGO_BIN_EXE_mouse-interpreter"))
            .arg(program)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(&input).unwrap();
        let output = child.wait_with_output().unwrap();

        let actual = String::from_utf8_lossy(&output.stdout);
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            failures.push(format!("{name}: {}", stderr.trim()));
        } else if actual != expected {
            failures.push(format!("{name}: output differs from {name}.out"));
        }
        checked += 1;
    }

    assert!(failures.is_empty(), "{}", failures.join("\n"));
    assert!(checked >= 40, "only {checked} examples were found");
}
