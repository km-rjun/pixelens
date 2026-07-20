//! Web-search URL construction for Pixelens.
//!
//! Ported from `origin/main` `crates/pixelens-core/src/actions/search.rs`
//! (Strategy C). The `ActionHandler`/`UrlLauncher` harness is dropped — this
//! crate only builds the URL; the daemon is responsible for opening it.

use urlencoding;

/// Build a Google search URL for the given free-text query.
///
/// The query is percent-encoded so that spaces, `&`, `#`, and other reserved
/// characters are safely embedded in the resulting URL.
pub fn build_search_url(text: &str) -> String {
    let encoded = urlencoding::encode(text);
    format!("https://www.google.com/search?q={}", encoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_expected_url_for_plain_text() {
        assert_eq!(
            build_search_url("rust programming"),
            "https://www.google.com/search?q=rust%20programming"
        );
    }

    #[test]
    fn encodes_ampersand_as_percent_26() {
        // '&' must be percent-encoded so it does not break the query string.
        let url = build_search_url("foo & bar");
        assert!(url.contains("%26"), "expected %26 for '&', got: {}", url);
        assert!(!url.contains('&'), "raw '&' should not appear in query");
    }

    #[test]
    fn handles_multiline_text() {
        let input = "line one\nline two";
        let url = build_search_url(input);
        // Newlines are encoded as %0A.
        assert!(url.starts_with("https://www.google.com/search?q="));
        assert!(url.contains("%0A"), "newline should be percent-encoded");
    }

    #[test]
    fn empty_input_produces_valid_prefix() {
        assert_eq!(build_search_url(""), "https://www.google.com/search?q=");
    }

    #[test]
    fn encodes_special_characters() {
        // '#' is encoded so it does not start a fragment.
        let url = build_search_url("rust#lang");
        assert!(url.contains("%23"), "expected %23 for '#', got: {}", url);
    }
}
