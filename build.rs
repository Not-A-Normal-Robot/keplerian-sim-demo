use std::fs;
use std::path::Path;

fn warn(msg: &str) {
    println!(
        "cargo:warning={}",
        msg.replace('%', "%25").replace('\n', "%0A")
    );
}

fn main() {
    // Only run when row-descs changes
    println!("cargo:rerun-if-changed=src/gui/celestials/row-descs");

    let base = Path::new("src/gui/celestials/row-descs");
    if !base.exists() {
        return;
    }

    let entries = match fs::read_dir(base) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("txt") {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("<file>");
        let ok = fs::read_to_string(&path);
        let s = match ok {
            Ok(v) => v,
            Err(e) => {
                warn(&format!("Failed to read {}: {}", name, e));
                continue;
            }
        };

        // Rule: first line exists
        if s.lines().next().map(|l| l.trim()).unwrap_or("").is_empty() {
            warn(&format!(
                "{}: first line (short description) is empty",
                name
            ));
        }

        // Rule: equation blocks must be separated by blank lines and use 4-space indentation
        for (i, line) in s.lines().enumerate() {
            // detect 4-space indented equation lines
            if line.starts_with("    ") {
                // no trailing dot
                if line.trim_end().ends_with('.') {
                    warn(&format!("{}:{}: equation line ends with '.'", name, i + 1));
                }
            }
            // detect "..where:" typo
            if line.trim_start().starts_with("..where:") {
                warn(&format!(
                    "{}:{}: use '...where:' (three dots), not '..where:'",
                    name,
                    i + 1
                ));
            }
            // detect micro sign vs mu: prefer U+03BC (mu)
            if line.contains('\u{00B5}') {
                warn(&format!(
                    "{}:{}: found micro sign U+00B5; prefer Greek small mu U+03BC",
                    name,
                    i + 1
                ));
            }
        }

        // Rule: ...where: exists
        if !s.contains("...where:") {
            warn(&format!("{}: missing '...where:' section", name));
        }
    }
}
