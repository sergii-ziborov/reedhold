//! Weavatrix-style budgets: file size, dual modules, crate layering.

use std::fs;
use std::path::{Path, PathBuf};

const MAX_FILE_LINES: usize = 300;
const MAX_FUNCTION_LINES: usize = 100;

#[test]
fn source_files_stay_within_the_budget() {
    let mut oversized = Vec::new();
    for path in rust_files(&workspace_root()) {
        let source = fs::read_to_string(&path).expect("read rust source");
        let lines = source.lines().count();
        if lines > MAX_FILE_LINES {
            oversized.push(format!("{} has {lines} lines", display(&path)));
        }
    }
    assert!(oversized.is_empty(), "{}", oversized.join("\n"));
}

#[test]
fn functions_stay_within_the_budget() {
    let mut oversized = Vec::new();
    for path in rust_files(&workspace_root()) {
        let source = fs::read_to_string(&path).expect("read rust source");
        for (name, lines) in function_spans(&source) {
            if lines > MAX_FUNCTION_LINES {
                oversized.push(format!("{}::{name} has {lines} lines", display(&path)));
            }
        }
    }
    assert!(oversized.is_empty(), "{}", oversized.join("\n"));
}

#[test]
fn rust_modules_use_one_unambiguous_layout() {
    let mut dual = Vec::new();
    for path in rust_files(&workspace_root()) {
        if path.file_stem().and_then(|stem| stem.to_str()) == Some("mod")
            || path.file_stem().and_then(|stem| stem.to_str()) == Some("lib")
        {
            continue;
        }
        let sibling_dir = path.with_extension("");
        if sibling_dir.is_dir() {
            dual.push(display(&path));
        }
    }
    assert!(dual.is_empty(), "dual module forms:\n{}", dual.join("\n"));
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn rust_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect(root, &mut files);
    files
}

fn collect(dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if should_skip(&path) {
                continue;
            }
            collect(&path, files);
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}

fn should_skip(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some("target" | ".git" | ".local")
    )
}

fn display(path: &Path) -> String {
    path.strip_prefix(workspace_root())
        .unwrap_or(path)
        .display()
        .to_string()
}

fn function_spans(source: &str) -> Vec<(String, usize)> {
    let mut spans = Vec::new();
    let mut lines = source.lines().enumerate();
    while let Some((index, line)) = lines.next() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("pub fn ")
            && !trimmed.starts_with("fn ")
            && !trimmed.starts_with("pub(crate) fn ")
        {
            continue;
        }
        let Some(name) = fn_name(trimmed) else {
            continue;
        };
        let start = index;
        let mut depth = brace_delta(line);
        if !line.contains('{') {
            continue;
        }
        if depth == 0 {
            spans.push((name, 1));
            continue;
        }
        for (end, next) in lines.by_ref() {
            depth += brace_delta(next);
            if depth <= 0 {
                spans.push((name, end - start + 1));
                break;
            }
        }
    }
    spans
}

fn fn_name(line: &str) -> Option<String> {
    let rest = line.split_once("fn ")?.1;
    let name = rest
        .split(|ch: char| ch == '(' || ch.is_whitespace())
        .next()?;
    if name.is_empty() {
        None
    } else {
        Some(name.to_owned())
    }
}

fn brace_delta(line: &str) -> i32 {
    let mut delta = 0;
    for ch in line.chars() {
        match ch {
            '{' => delta += 1,
            '}' => delta -= 1,
            _ => {}
        }
    }
    delta
}
