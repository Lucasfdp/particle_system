//! Reading the particle count from the command line (§ 1).
//!
//! Plain numbers always work (`1000000`, `1_000_000`), plus shortcuts for testing
//! (`500k`, `3m`, `1.5m`, `1e6`, `2^20`) and a few jokes (`pi`, `42`, `lots`, `max`,
//! `googol`). `0` is allowed on purpose: the app opens an empty window for it and
//! never creates a particle buffer (a zero-sized Vulkan buffer is invalid).

use std::time::{SystemTime, UNIX_EPOCH};

/// The most particles we accept. One 1D dispatch holds at most 65,535 workgroups
/// (the minimum Vulkan guarantees, § 2); at 256 per workgroup that's this many.
/// § 16 replaces it with a number based on the actual GPU.
pub const MAX_COUNT: u32 = 65_535 * 256;

/// A count the app can use, and an optional line explaining how it was read.
#[derive(Debug, Clone, PartialEq)]
pub struct Count {
    pub value: u32,
    pub note: Option<String>,
}

impl Count {
    fn plain(value: u32) -> Self {
        Count { value, note: None }
    }

    fn noted(value: u32, note: impl Into<String>) -> Self {
        Count {
            value,
            note: Some(note.into()),
        }
    }
}

/// Reads `text` as a particle count. `Err` holds a message for the user.
pub fn parse_count(text: &str) -> Result<Count, String> {
    let lower = text.trim().to_ascii_lowercase();
    let word = lower.replace('_', "");

    match word.as_str() {
        "pi" => return Ok(Count::noted(3_141_592, "pi → 3,141,592 particles")),
        "42" => {
            return Ok(Count::noted(
                4_200_000,
                "42 → 4,200,000 (the answer, scaled up)",
            ));
        }
        "lots" => {
            let value = random_between(1_000_000, 5_000_000);
            return Ok(Count::noted(
                value,
                format!("lots → {}", with_commas(value)),
            ));
        }
        "max" => {
            return Ok(Count::noted(
                MAX_COUNT,
                format!(
                    "max → {} (§ 16 will base this on your GPU)",
                    with_commas(MAX_COUNT)
                ),
            ));
        }
        "googol" => {
            return Ok(Count::noted(
                MAX_COUNT,
                format!("googol? Nice try. Clamped to {}", with_commas(MAX_COUNT)),
            ));
        }
        _ => {}
    }

    let value = number(&word).ok_or_else(|| format!("'{text}' isn't a particle count"))?;
    if value > f64::from(MAX_COUNT) {
        return Err(format!(
            "{} particles is too many (max {})",
            with_commas_f64(value),
            with_commas(MAX_COUNT)
        ));
    }
    Ok(Count::plain(value as u32))
}

/// A whole, non-negative number written as digits, `1.5m`, `500k`, `1e6` or `2^20`.
/// `None` for anything else, negatives and fractions included.
fn number(word: &str) -> Option<f64> {
    let value = if let Some((base, exponent)) = word.split_once('^') {
        let base: u64 = base.parse().ok()?;
        let exponent: u32 = exponent.parse().ok()?;
        // Too big for u64 (10^30) still means "too many", not "not a number".
        base.checked_pow(exponent)
            .map_or(f64::INFINITY, |v| v as f64)
    } else if let Some(digits) = word.strip_suffix('k') {
        decimal(digits)? * 1_000.0
    } else if let Some(digits) = word.strip_suffix('m') {
        decimal(digits)? * 1_000_000.0
    } else {
        decimal(word)? // also reads `1e6`
    };
    let whole = value == f64::INFINITY || (value >= 0.0 && value.fract() == 0.0);
    whole.then_some(value)
}

/// A plain decimal (`3`, `1.5`, `1e6`), without signs, `inf` or `nan`.
fn decimal(text: &str) -> Option<f64> {
    let digits_only = text
        .chars()
        .all(|c| c.is_ascii_digit() || c == '.' || c == 'e');
    if text.is_empty() || !digits_only || !text.starts_with(|c: char| c.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

/// A different number in `low..=high` each run (seeded from the clock). Not for
/// anything serious — no external crate needed for a joke.
fn random_between(low: u32, high: u32) -> u32 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos() as u64);
    // splitmix64: one step turns a clock reading into well-mixed bits.
    let mut x = nanos.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^= x >> 31;
    low + (x % u64::from(high - low + 1)) as u32
}

/// `1234567` → `"1,234,567"`.
pub fn with_commas(value: u32) -> String {
    commas(u64::from(value))
}

/// For "too many" messages, where the number can be anything up to infinity.
fn with_commas_f64(value: f64) -> String {
    if value < 1e15 {
        commas(value as u64)
    } else {
        "way too many".to_string()
    }
}

/// Any whole number with commas (dispatch sizes can pass `u32::MAX`).
pub fn commas(value: u64) -> String {
    let digits = value.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value(text: &str) -> u32 {
        parse_count(text).unwrap().value
    }

    #[test]
    fn plain_numbers_and_underscores() {
        assert_eq!(value("1000000"), 1_000_000);
        assert_eq!(value("1_500_000"), 1_500_000);
        assert_eq!(value(" 7 "), 7);
    }

    #[test]
    fn shortcuts() {
        assert_eq!(value("500k"), 500_000);
        assert_eq!(value("3m"), 3_000_000);
        assert_eq!(value("3M"), 3_000_000);
        assert_eq!(value("1.5m"), 1_500_000);
        assert_eq!(value("1e6"), 1_000_000);
        assert_eq!(value("2^20"), 1_048_576);
    }

    #[test]
    fn jokes() {
        assert_eq!(value("pi"), 3_141_592);
        assert_eq!(value("42"), 4_200_000);
        assert_eq!(value("max"), MAX_COUNT);
        assert_eq!(value("googol"), MAX_COUNT);
        let lots = value("lots");
        assert!((1_000_000..=5_000_000).contains(&lots));
    }

    #[test]
    fn zero_is_allowed() {
        assert_eq!(value("0"), 0);
    }

    #[test]
    fn junk_negatives_and_fractions_are_errors() {
        for bad in [
            "banana", "", "-5", "1.5", "1.2345k", "k", "e6", "inf", "nan", "3mm",
        ] {
            assert!(parse_count(bad).is_err(), "{bad:?} should be rejected");
        }
    }

    #[test]
    fn too_many_is_an_error() {
        assert!(parse_count("17m").is_err());
        assert!(parse_count("2^40").is_err());
        assert!(parse_count("10^30").is_err());
        assert!(parse_count("1e300").is_err());
    }

    #[test]
    fn commas() {
        assert_eq!(with_commas(0), "0");
        assert_eq!(with_commas(999), "999");
        assert_eq!(with_commas(1_000), "1,000");
        assert_eq!(with_commas(3_141_592), "3,141,592");
    }
}
