//! Parses the channel table out of `docs/THREAT-MODEL.md` at test time. This
//! deliberately does not keep its own copy of the table — a hardcoded copy
//! is exactly how "every channel maps to a test" would silently stop
//! tracking the document it is supposed to guard (T16's own acceptance
//! criterion names this failure mode directly).

use std::path::Path;

pub(super) struct ChannelRow {
    pub(super) id: String,
    /// `Some(name)` for a named test, `None` for a literal `GAP`.
    pub(super) check: Option<String>,
}

pub(super) fn parse(threat_model_text: &str) -> Vec<ChannelRow> {
    threat_model_text
        .lines()
        .filter(|line| line.trim_start().starts_with("| CH-"))
        .filter_map(parse_row)
        .collect()
}

fn parse_row(line: &str) -> Option<ChannelRow> {
    let columns: Vec<&str> = line
        .trim()
        .trim_matches('|')
        .split('|')
        .map(str::trim)
        .collect();
    let id = (*columns.first()?).to_string();
    let check_raw = *columns.last()?;
    let check = if check_raw == "GAP" {
        None
    } else {
        Some(check_raw.trim_matches('`').to_string())
    };
    Some(ChannelRow { id, check })
}

/// The `### CH-XX — ...` sub-headings under `## Gaps` — every id documented
/// there as having no automated check, and therefore exempt from needing a
/// named test.
pub(super) fn gap_ids(threat_model_text: &str) -> Vec<String> {
    let Some(gaps_start) = threat_model_text.find("\n## Gaps") else {
        return Vec::new();
    };
    threat_model_text[gaps_start..]
        .lines()
        .filter_map(|line| {
            let heading = line.trim().strip_prefix("### ")?;
            if !heading.starts_with("CH-") {
                return None;
            }
            // Consume the id itself (letters, digits, hyphen) and stop at
            // the space before the em dash and title — "CH-15" contains a
            // hyphen too, so this cannot split on '-' generically.
            let id: String = heading
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
                .collect();
            Some(id)
        })
        .collect()
}

pub(super) fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()))
}
