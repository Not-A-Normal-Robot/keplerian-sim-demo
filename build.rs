use std::fs;
use std::path::Path;
use std::str::Lines;

fn warn(msg: &str) {
    println!(
        "cargo:warning={}",
        msg.replace('%', "%25").replace('\n', "%0A")
    );
}

fn main() {
    check_row_descs_format();
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RowDescSection {
    ShortDesc,
    LongDesc,
    Equation,
    Final,
}

struct RowDescState<'a> {
    lines: Lines<'a>,
    filename: &'a str,
    line_num: usize,
    section: RowDescSection,
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
    let lines = text.lines();
    let mut state = RowDescState {
        filename,
        lines,
        line_num: 0,
        section: RowDescSection::ShortDesc,
    };

    loop {
        let should_break = check_row_desc_format_inner(&mut state);

        if should_break {
            break;
        }
    }
}

/// Returns whether or not to break out of the loop
fn check_row_desc_format_inner(state: &mut RowDescState) -> bool {
    let line = state.lines.next();
    state.line_num += 1;

    check_discouraged_chars(line, state);

    match state.section {
        RowDescSection::ShortDesc => check_short_desc(line, state),
        RowDescSection::LongDesc => check_long_desc(line, state),
        RowDescSection::Equation => check_equation(line, state),
        RowDescSection::Final => line.is_none(),
    }
}

fn check_short_desc(line: Option<&str>, state: &mut RowDescState) -> bool {
    state.section = RowDescSection::LongDesc;
    let filename = state.filename;

    let first_line = match line {
        Some(l) => l,
        None => {
            warn(&format!("{filename}: empty file"));
            return true;
        }
    };
    // Math variable name must be in parens, e.g.:
    // Side length (s) of the square.
    let open_idx = match first_line.find('(') {
        Some(i) => i,
        None => {
            warn(&format!(
                "{filename}:1: Short desc missing var name in parentheses"
            ));
            return false;
        }
    };
    let close_idx = match first_line.find(')') {
        Some(i) => i,
        None => {
            warn(&format!(
                "{filename}:1: Short desc missing var name in parentheses"
            ));
            return false;
        }
    };

    if open_idx > close_idx {
        warn(&format!("{filename}:1: Swapped parentheses!"));
    }

    return false;
}

fn check_long_desc(line: Option<&str>, state: &mut RowDescState) -> bool {
    let line = match line {
        Some(l) => l,
        None => {
            return true;
        }
    };

    if line.trim().is_empty() {
        state.section = RowDescSection::Equation;
    }

    return false;
}

fn check_equation(line: Option<&str>, state: &mut RowDescState) -> bool {
    enum LineKind {
        Condition,
        Equation,
    }

    let line = match line {
        Some(l) => l,
        None => {
            return true;
        }
    };

    let filename = state.filename;
    let line_num = state.line_num;

    let indentation = line.chars().take_while(|c| c.is_whitespace()).count();

    let kind = match indentation {
        4 => LineKind::Equation,
        2 => LineKind::Condition,
        0 => {
            // End of equations
            state.section = RowDescSection::Final;
            return false;
        }
        x if x > 2 => {
            warn(&format!(
                "{filename}:{line_num}: {x}-char indentation, \
                interpreting as equation (4 chars) instead; \
                use 2 chars instead for condition"
            ));
            LineKind::Equation
        }
        x => {
            warn(&format!(
                "{filename}:{line_num}: {x}-char indentation, \
                interpreting as condition (2 chars) instead; \
                use 4 chars instead for equation"
            ));
            LineKind::Condition
        }
    };

    let open_parens = line.matches('(').count();
    let close_parens = line.matches(')').count();

    if open_parens != close_parens {
        warn(&format!("{filename}:{line_num}: mismatched parentheses"));
    }

    match kind {
        LineKind::Condition => {
            if !line.contains(':') {
                warn(&format!(
                    "{filename}:{line_num}: condition expected, no `:` found"
                ));
            }
        }
        LineKind::Equation => {
            if !line.contains('=') {
                warn(&format!(
                    "{filename}:{line_num}: equation expected, no `=` found"
                ));
            }

            if line.trim_end().ends_with('.') {
                warn(&format!(
                    "{filename}:{line_num}: equations should not end in full stop"
                ));
            }
        }
    }

    return false;
}

fn check_discouraged_chars(line: Option<&str>, state: &RowDescState) {
    struct DiscouragedChar<'a> {
        discouraged: char,
        discouraged_name: &'a str,
        suggested: char,
        suggested_name: &'a str,
        section_filter: Option<RowDescSection>,
    }

    const DISCOURAGED_CHARS: [DiscouragedChar; 6] = [
        DiscouragedChar {
            discouraged: 'µ',
            discouraged_name: "U+00B5 MICRO SIGN",
            suggested: 'μ',
            suggested_name: "U+03BC GREEK SMALL LETTER MU",
            section_filter: None,
        },
        DiscouragedChar {
            discouraged: '*',
            discouraged_name: "U+002A ASTERISK",
            suggested: '⋅',
            suggested_name: "U+22C5 DOT OPERATOR",
            section_filter: None,
        },
        DiscouragedChar {
            discouraged: '∙',
            discouraged_name: "U+2219 BULLET OPERATOR",
            suggested: '⋅',
            suggested_name: "U+22C5 DOT OPERATOR",
            section_filter: None,
        },
        DiscouragedChar {
            discouraged: '·',
            discouraged_name: "U+00B7 MIDDLE DOT",
            suggested: '⋅',
            suggested_name: "U+22C5 DOT OPERATOR",
            section_filter: None,
        },
        DiscouragedChar {
            discouraged: '•',
            discouraged_name: "U+2022 BULLET",
            suggested: '⋅',
            suggested_name: "U+22C5 DOT OPERATOR",
            section_filter: None,
        },
        DiscouragedChar {
            discouraged: '\'',
            discouraged_name: "U+0027 APOSTROPHE",
            suggested: '′',
            suggested_name: "U+2032 PRIME",
            section_filter: Some(RowDescSection::Equation),
        },
    ];

    let line = match line {
        Some(l) => l,
        None => return,
    };
    let line_num = state.line_num;
    let filename = state.filename;

    for (index, char) in line.chars().enumerate() {
        let discouraged_char = DISCOURAGED_CHARS.iter().find(|dc| dc.discouraged == char);

        let discouraged_char = match discouraged_char {
            Some(dc) => dc,
            None => continue,
        };

        let DiscouragedChar {
            discouraged,
            discouraged_name,
            suggested,
            suggested_name,
            section_filter,
        } = discouraged_char;

        if let Some(section_filter) = section_filter
            && *section_filter != state.section
        {
            continue;
        }

        let char_num = index + 1;

        warn(&format!(
            "{filename}:{line_num}:{char_num}: \
            used discouraged char {discouraged_name} ({discouraged}); \
            use {suggested_name} ({suggested}) instead"
        ));
    }
}
