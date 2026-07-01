use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

// Layer 1 — hardcoded, not user-editable. If the frontmost app matches,
// the poll cycle skips before the clipboard is ever read.
const BLOCKED_BUNDLE_IDS: &[&str] = &[
    "com.1password.1password",
    "com.bitwarden.desktop",
    "org.keepassxc.keepassxc",
    "com.apple.keychainaccess",
    "com.apple.Security",
    "com.strongbox.mac",
    "com.dashlane.dashlanephoneapp",
    "com.lastpass.LastPass",
];

// ponytail: narrow allowlist of apps OTP codes plausibly come from, so a
// random 6-8 digit clip (a PIN, a zip code) isn't blocked everywhere.
const OTP_SOURCE_BUNDLE_IDS: &[&str] = &[
    "com.apple.MobileSMS",
    "com.authy.authy",
    "com.google.authenticator",
];

pub fn is_blocked_app(bundle_id: &str) -> bool {
    BLOCKED_BUNDLE_IDS.contains(&bundle_id)
}

static AWS_KEY_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"AKIA[0-9A-Z]{16}").unwrap());
static PRIVATE_KEY_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"-----BEGIN [A-Z0-9 ]*PRIVATE KEY-----").unwrap());
static JWT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+$").unwrap());
static OTP_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d{6,8}$").unwrap());
static CARD_CANDIDATE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\d[\d -]{11,22}\d").unwrap());

fn shannon_entropy(s: &str) -> f64 {
    let len = s.len() as f64;
    let mut counts: HashMap<char, u32> = HashMap::new();
    for c in s.chars() {
        *counts.entry(c).or_insert(0) += 1;
    }
    counts
        .values()
        .map(|&count| {
            let p = count as f64 / len;
            -p * p.log2()
        })
        .sum()
}

// ponytail: entropy is checked per whitespace-delimited token (an API key
// is one token), not over the whole clip — scanning full text would flag
// ordinary prose. Ceiling: base64 blobs, hashes and UUIDs can still trip
// this; revisit once the content classifier can exempt known-safe types.
fn has_high_entropy_token(content: &str) -> bool {
    content
        .split_whitespace()
        .any(|token| token.len() > 32 && shannon_entropy(token) > 4.5)
}

fn luhn_valid(digits: &str) -> bool {
    let digits: Vec<u32> = digits.chars().filter_map(|c| c.to_digit(10)).collect();
    if digits.len() < 13 || digits.len() > 19 {
        return false;
    }
    let sum: u32 = digits
        .iter()
        .rev()
        .enumerate()
        .map(|(i, &d)| {
            if i % 2 == 1 {
                let doubled = d * 2;
                if doubled > 9 {
                    doubled - 9
                } else {
                    doubled
                }
            } else {
                d
            }
        })
        .sum();
    sum % 10 == 0
}

fn contains_credit_card(content: &str) -> bool {
    CARD_CANDIDATE_RE.find_iter(content).any(|m| {
        let digits: String = m.as_str().chars().filter(|c| c.is_ascii_digit()).collect();
        luhn_valid(&digits)
    })
}

fn is_bip39_phrase(content: &str) -> bool {
    let words: Vec<&str> = content.split_whitespace().collect();
    if words.len() != 12 && words.len() != 24 {
        return false;
    }
    let wordlist = bip39::Language::English.word_list();
    words
        .iter()
        .all(|w| wordlist.contains(&w.to_lowercase().as_str()))
}

/// Layer 3 — content pattern detection. Only called after Layers 1 and 2 pass.
pub fn is_sensitive(content: &str, bundle_id: &str) -> bool {
    let trimmed = content.trim();

    AWS_KEY_RE.is_match(content)
        || PRIVATE_KEY_RE.is_match(content)
        || JWT_RE.is_match(trimmed)
        || has_high_entropy_token(content)
        || contains_credit_card(content)
        || (OTP_SOURCE_BUNDLE_IDS.contains(&bundle_id) && OTP_RE.is_match(trimmed))
        || is_bip39_phrase(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_password_manager_apps() {
        assert!(is_blocked_app("com.1password.1password"));
        assert!(!is_blocked_app("com.apple.Terminal"));
    }

    #[test]
    fn detects_aws_key() {
        assert!(is_sensitive("AKIAABCDEFGHIJKLMNOP", "com.apple.Terminal"));
    }

    #[test]
    fn detects_private_key_header() {
        assert!(is_sensitive(
            "-----BEGIN RSA PRIVATE KEY-----\nMIIEow==",
            "com.apple.Terminal"
        ));
    }

    #[test]
    fn detects_jwt() {
        let jwt = "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U";
        assert!(is_sensitive(jwt, "com.apple.Terminal"));
    }

    #[test]
    fn detects_credit_card() {
        assert!(is_sensitive("4111 1111 1111 1111", "com.apple.Terminal"));
    }

    #[test]
    fn detects_otp_from_sms_only() {
        assert!(is_sensitive("482913", "com.apple.MobileSMS"));
        assert!(!is_sensitive("482913", "com.apple.Terminal"));
    }

    #[test]
    fn plain_text_is_not_flagged() {
        assert!(!is_sensitive(
            "just a normal sentence copied from an email",
            "com.apple.Terminal"
        ));
    }

    #[test]
    fn short_random_token_is_not_flagged() {
        assert!(!is_sensitive("hello world", "com.apple.Terminal"));
    }
}
