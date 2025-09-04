use std::fs;
use std::path::Path;

fn warn(msg: &str) {
    println!(
        "cargo:warning={}",
        msg.replace('%', "%25").replace('\n', "%0A")
    );
}

fn main() {
    check_row_descs_format();
}

fn check_row_descs_format() {
    const ROW_DESCS_PATH_STR: &str = "src/gui/celestials/row-descs";
    println!("cargo:rerun-if-changed={ROW_DESCS_PATH_STR}");

    let base = Path::new(ROW_DESCS_PATH_STR);
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
        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("<file>");
        let result = fs::read_to_string(&path);
        let text = match result {
            Ok(v) => v,
            Err(e) => {
                warn(&format!("Failed to read {}: {}", filename, e));
                continue;
            }
        };

        check_row_desc_format(filename, &text);
    }
}

fn check_row_desc_format(filename: &str, text: &str) {
    // Rule: first line exists
    if text
        .lines()
        .next()
        .map(|l| l.trim())
        .unwrap_or("")
        .is_empty()
    {
        warn(&format!(
            "{}: first line (short description) is empty",
            filename
        ));
    }

    // Rule: equation blocks must be separated by blank lines and use 4-space indentation
    for (i, line) in text.lines().enumerate() {
        // detect 4-space indented equation lines
        if line.starts_with("    ") {
            // no trailing dot
            if line.trim_end().ends_with('.') {
                warn(&format!(
                    "{}:{}: equation line ends with '.'",
                    filename,
                    i + 1
                ));
            }
        }
        // detect micro sign vs mu: prefer U+03BC (mu)
        if line.contains('\u{00B5}') {
            warn(&format!(
                "{}:{}: found micro sign U+00B5; prefer Greek small mu U+03BC",
                filename,
                i + 1
            ));
        }
    }

    // Rule: ...where: exists
    if !text.contains("...where:") {
        warn(&format!("{}: missing '...where:' section", filename));
    }
}
