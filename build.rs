fn warn(msg: &str) {
    println!(
        "cargo:warning={}",
        msg.replace('%', "%25").replace('\n', "%0A")
    );
}

fn main() {
    row_descs::check();
    export_keplerian_sim_version();
    presets::build();
}

mod row_descs {
    use std::{fs, path::Path, str::Lines};

    use super::warn;
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

    pub(super) fn check() {
        const ROW_DESCS_PATH_STR: &str = "src/gui/celestials/row_descs";
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

    fn is_string_snake_case(string: &str) -> bool {
        let mut is_prev_underscore = false;
        for char in string.chars() {
            if char.is_uppercase() {
                return false;
            }

            let is_curr_underscore = char == '_';

            if is_curr_underscore && is_prev_underscore {
                return false;
            }

            is_prev_underscore = is_curr_underscore
        }

        return true;
    }

    fn check_row_desc_format(filename: &str, text: &str) {
        if !filename.is_ascii() {
            warn(&format!("{filename}: non-ascii file name"));
        } else if !filename.contains("EXAMPLE") && !is_string_snake_case(filename) {
            warn(&format!("{filename}: file name not using snake_case"));
        }

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
}

fn export_keplerian_sim_version() {
    // Attempt to read resolved dependency versions via cargo metadata and
    // publish the keplerian_sim version as an environment variable so it can
    // be accessed at runtime via env! or option_env!.
    match cargo_metadata::MetadataCommand::new().exec() {
        Ok(metadata) => {
            let pkg_map = metadata
                .packages
                .into_iter()
                .map(|p| (p.name.clone(), p.version.to_string()))
                .collect::<std::collections::BTreeMap<_, _>>();

            if let Some(ver) = pkg_map.get("keplerian_sim") {
                println!("cargo:rustc-env=KEPLERIAN_SIM_VERSION={}", ver);
            } else {
                println!("cargo:rustc-env=KEPLERIAN_SIM_VERSION=unknown");
            }
        }
        Err(e) => {
            warn(&format!("failed to query cargo metadata: {}", e));
            println!("cargo:rustc-env=KEPLERIAN_SIM_VERSION=unknown");
        }
    }
}

mod presets {
    use std::{
        borrow::Cow,
        fmt::Display,
        fs::{self, File},
        io::Write,
    };

    use toml::{
        Spanned,
        de::{DeArray, DeInteger, DeTable, DeValue},
    };

    const PRESETS_TOML_PATH: &str = "src/sim/presets.toml";
    const OUTPUT_PATH: &str = "src/sim/presets.rs";

    const KEY_NAME: &str = "name";
    const KEY_DOCNAME: &str = "docname";
    const KEY_DESC: &str = "description";
    const KEY_MASS: &str = "mass";
    const KEY_RADIUS: &str = "radius";
    const KEY_APOAPSIS: &str = "apoapsis";
    const KEY_ECCENTRICITY: &str = "eccentricity";
    const KEY_PERIAPSIS: &str = "periapsis";
    const KEY_INCLINATION: &str = "inclination";
    const KEY_ARG_PE: &str = "arg_pe";
    const KEY_LONG_ASC_NODE: &str = "long_asc_node";
    const KEY_MEAN_ANOMALY: &str = "mean_anomaly";
    const KEY_COLOR: &str = "color";

    struct BodyCreator<'a> {
        fn_name: &'a str,
        name: &'a str,
        docname: &'a str,
        desc: Option<&'a str>,
        mass: f64,
        radius: f64,
        eccentricity: f64,
        periapsis: f64,
        inclination: f64,
        arg_pe: f64,
        long_asc_node: f64,
        mean_anomaly: f64,
        color: [u8; 3],
    }

    pub(super) fn build() {
        println!("cargo:rerun-if-changed={PRESETS_TOML_PATH}");
        let mut output_file =
            File::create(OUTPUT_PATH).expect("failed to initialize output file writer");
        print_header(&mut output_file);

        let presets_string =
            fs::read_to_string(PRESETS_TOML_PATH).expect("failed to read from presets file");
        let table = DeTable::parse(&presets_string).expect("failed to parse presets file as table");
        for (fn_name, value) in table.get_ref() {
            process_entry(fn_name, value, &mut output_file);
        }
    }

    fn print_header(file: &mut File) {
        file.write_all(
            b"//! Generated by build.rs::presets\n\
            #![allow(clippy::excessive_precision)]\n\
            use super::body::Body;\n\
            use keplerian_sim::Orbit;\n\
            use three_d::Srgba;\n",
        )
        .expect("failed to write to output file");
    }

    fn process_entry(
        fn_name: &Spanned<Cow<'_, str>>,
        value: &Spanned<DeValue<'_>>,
        file: &mut File,
    ) {
        let fn_name = fn_name.get_ref();
        let value = value.get_ref();
        let map = match value {
            DeValue::Table(map) => map,
            v => {
                panic!("preset builder: parsing {fn_name}: expected table, found {v:?}");
            }
        };

        // let Some(name) = table.get(KEY_NAME).and_then(|v| v.get_ref().as_str()) else {
        //     panic!("preset builder: {fn_name} does not contain {KEY_NAME} of type string");
        // };
        let name = get_str_required(map, fn_name, KEY_NAME);
        let docname = get_str_optional(map, fn_name, KEY_DOCNAME).unwrap_or(name);
        let desc = get_str_optional(map, fn_name, KEY_DESC);
        let mass = get_float_required(map, fn_name, KEY_MASS);
        let radius = get_float_required(map, fn_name, KEY_RADIUS);

        let apoapsis = get_float_optional(map, fn_name, KEY_APOAPSIS);
        let eccentricity = get_float_optional(map, fn_name, KEY_ECCENTRICITY);

        enum EccentricityDefiner {
            Eccentricity(f64),
            Apoapsis(f64),
        }

        let ecc_definer = match (apoapsis, eccentricity) {
            (None, None) => panic!(
                "preset builder: {fn_name}: missing either apoapsis or eccentricity (pick one and define)"
            ),
            (None, Some(e)) => EccentricityDefiner::Eccentricity(e),
            (Some(ap), None) => EccentricityDefiner::Apoapsis(ap),
            (Some(_), Some(_)) => panic!(
                "preset builder: {fn_name}: defining both apoapsis and eccentricity is disallowed"
            ),
        };

        let periapsis = get_float_required(map, fn_name, KEY_PERIAPSIS);

        let eccentricity = match ecc_definer {
            EccentricityDefiner::Eccentricity(e) => e,
            EccentricityDefiner::Apoapsis(apoapsis) => {
                (apoapsis - periapsis) / (apoapsis + periapsis)
            }
        };

        let inclination = get_float_required(map, fn_name, KEY_INCLINATION).to_radians();
        let arg_pe = get_float_required(map, fn_name, KEY_ARG_PE).to_radians();
        let long_asc_node = get_float_required(map, fn_name, KEY_LONG_ASC_NODE).to_radians();
        let mean_anomaly = get_float_required(map, fn_name, KEY_MEAN_ANOMALY).to_radians();

        let color = get_srgb_required(map, fn_name, KEY_COLOR);

        let creator = BodyCreator {
            fn_name,
            name,
            docname,
            desc,
            mass,
            radius,
            eccentricity,
            periapsis,
            inclination,
            arg_pe,
            long_asc_node,
            mean_anomaly,
            color,
        };

        let code = meta_create_body(&creator);

        file.write_all(&code.as_bytes())
            .expect("failed to write to output file");
    }

    fn meta_create_body(creator: &BodyCreator) -> String {
        let BodyCreator {
            fn_name,
            name,
            docname,
            desc,
            mass,
            radius,
            eccentricity,
            periapsis,
            inclination,
            arg_pe,
            long_asc_node,
            mean_anomaly,
            color,
        } = creator;
        let [color_r, color_g, color_b] = color;
        let desc = match desc {
            Some(d) => format!(", {d}"),
            None => String::new(),
        };

        format!(
            "
/// Returns {docname}{desc}.
///
/// `parent_mu`: The gravitational parameter of the parent body, if any.
/// If None, the celestial body will not be placed in an orbit.
pub(crate) fn {fn_name}(parent_mu: Option<f64>) -> Body {{
    let orbit = parent_mu.map(|mu| {{
        Orbit::new(
            {eccentricity:.20e},
            {periapsis:.20e},
            {inclination:.20e},
            {arg_pe:.20e},
            {long_asc_node:.20e},
            {mean_anomaly:.20e},
            mu,
        )
    }});

    Body {{
        name: String::from(\"{name}\"),
        mass: {mass:.20e},
        radius: {radius:.20e},
        orbit,
        color: Srgba::new_opaque({color_r}, {color_g}, {color_b}),
    }}
}}"
        )
    }

    fn expect_exists<'a>(
        map: &'a DeTable,
        fn_name: (impl AsRef<str> + Display),
        key_name: &str,
    ) -> &'a Spanned<DeValue<'a>> {
        let Some(val) = map.get(key_name) else {
            panic!("preset builder: {fn_name}: missing required field {key_name}");
        };
        val
    }

    fn get_str_required<'a>(
        map: &'a DeTable,
        fn_name: (impl AsRef<str> + Display),
        key_name: &str,
    ) -> &'a str {
        expect_str(
            expect_exists(map, fn_name.as_ref(), key_name),
            fn_name,
            key_name,
        )
    }

    fn get_str_optional<'a>(
        map: &'a DeTable,
        fn_name: (impl AsRef<str> + Display),
        key_name: &str,
    ) -> Option<&'a str> {
        let val = map.get(key_name)?;
        Some(expect_str(val, fn_name, key_name))
    }

    fn expect_str<'a>(
        val: &'a Spanned<DeValue<'a>>,
        fn_name: (impl AsRef<str> + Display),
        key_name: &str,
    ) -> &'a str {
        let Some(val) = val.get_ref().as_str() else {
            panic!("preset builder: {fn_name}: expected field {key_name} to be string");
        };
        val
    }

    fn get_float_required(
        map: &DeTable,
        fn_name: (impl AsRef<str> + Display),
        key_name: &str,
    ) -> f64 {
        expect_float(expect_exists(map, &fn_name, key_name), fn_name, key_name)
    }

    fn get_float_optional(
        map: &DeTable,
        fn_name: (impl AsRef<str> + Display),
        key_name: &str,
    ) -> Option<f64> {
        let val = map.get(key_name)?;
        Some(expect_float(val, fn_name, key_name))
    }

    fn expect_float(
        val: &Spanned<DeValue<'_>>,
        fn_name: (impl AsRef<str> + Display),
        key_name: &str,
    ) -> f64 {
        let Some(val) = val.get_ref().as_float() else {
            panic!("preset builder: {fn_name}: expected field {key_name} to be float");
        };
        let res = match val.as_str().parse() {
            Ok(f) => f,
            Err(e) => {
                panic!("preset builder: {fn_name}: failed to parse field {key_name} as float: {e}")
            }
        };
        res
    }

    fn get_srgb_required(
        map: &DeTable,
        fn_name: (impl AsRef<str> + Display),
        key_name: &str,
    ) -> [u8; 3] {
        expect_srgb(expect_exists(map, &fn_name, key_name), fn_name, key_name)
    }

    fn expect_srgb(
        val: &Spanned<DeValue<'_>>,
        fn_name: (impl AsRef<str> + Display),
        key_name: &str,
    ) -> [u8; 3] {
        match val.get_ref() {
            DeValue::Integer(de_integer) => srgb_from_int(de_integer, fn_name, key_name),
            DeValue::Array(de_array) => srgb_from_array(de_array, fn_name, key_name),
            _ => {
                panic!(
                    "preset builder: {fn_name}: expected field {key_name} \
                    to be either a 3-element integer array or an integer"
                );
            }
        }
    }

    fn srgb_from_int(
        val: &DeInteger<'_>,
        fn_name: (impl AsRef<str> + Display),
        key_name: &str,
    ) -> [u8; 3] {
        const SRGB_BITS: u32 = u8::BITS * 3;
        const SRGB_MAX_VALUE: u32 = (1 << SRGB_BITS) - 1;

        let val = match u32::from_str_radix(val.as_str(), val.radix()) {
            Ok(v) => v,
            Err(e) => {
                panic!(
                    "preset builder: {fn_name}: expected integer {key_name} to fit in 24 bits, got error {e}"
                );
            }
        };

        if val > SRGB_MAX_VALUE {
            panic!(
                "preset builder: {fn_name}: expected integer {key_name} to fit in 24 bits\n\
                ...max 24-bit number: {SRGB_MAX_VALUE} = 0x{SRGB_MAX_VALUE:X}\n\
                ...got value: {val} = 0x{val:X}"
            );
        }

        let [_, rgb @ ..] = val.to_be_bytes();
        rgb
    }

    fn srgb_from_array(
        val: &DeArray<'_>,
        fn_name: (impl AsRef<str> + Display),
        key_name: &str,
    ) -> [u8; 3] {
        if val.len() != 3 {
            panic!(
                "preset builder: {fn_name}: expected array {key_name} to have 3 elements, got {}",
                val.len()
            );
        }

        core::array::from_fn(|i| {
            let Some(val) = val[i].get_ref().as_integer() else {
                panic!("preset builder: {fn_name}: expected {key_name}[{i}] to be u8");
            };

            let val = match u8::from_str_radix(val.as_str(), val.radix()) {
                Ok(v) => v,
                Err(e) => panic!(
                    "preset builder: {fn_name}: expected {key_name}[{i}] to fit in u8, got error {e}"
                ),
            };
            val
        })
    }
}
