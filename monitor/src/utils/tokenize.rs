//! Shell-like tokenizer for skill name extraction.
//!
//! This module provides utilities for extracting skill names from history.jsonl
//! entries where skills are invoked with a leading `/`.
//!
//! # Overview
//!
//! The tokenizer handles shell-like quoting rules to correctly parse skill names
//! from commands like:
//! - `/commit -m "fix: update docs"` -> `commit`
//! - `/"my skill" arg1` -> `"my skill"` (preserves quotes for quoted skill names)
//! - `/sdd:plan` -> `sdd:plan`
//!
//! # Quoting Rules
//!
//! - Double quotes (`"`) group characters together
//! - Single quotes (`'`) group characters together
//! - Backslash (`\`) escapes the next character within quotes
//! - Unquoted tokens are terminated by whitespace
//!
//! # Example
//!
//! ```
//! use vibetea_monitor::utils::tokenize::extract_skill_name;
//!
//! assert_eq!(extract_skill_name("/commit -m 'message'"), Some("commit".to_string()));
//! assert_eq!(extract_skill_name("/\"my skill\" arg1"), Some("\"my skill\"".to_string()));
//! assert_eq!(extract_skill_name("/sdd:plan"), Some("sdd:plan".to_string()));
//! assert_eq!(extract_skill_name("not a skill"), None);
//! ```

/// Extracts the skill name from a command string.
///
/// The command must start with `/` followed by the skill name. The skill name
/// can be:
/// - An unquoted token (terminated by whitespace)
/// - A double-quoted string (preserves the quotes in output)
/// - A single-quoted string (preserves the quotes in output)
///
/// # Arguments
///
/// * `command` - The command string to parse, e.g., `/commit -m "message"`
///
/// # Returns
///
/// - `Some(skill_name)` if a valid skill name was found
/// - `None` if the input is empty, doesn't start with `/`, or has no valid token
///
/// # Examples
///
/// ```
/// use vibetea_monitor::utils::tokenize::extract_skill_name;
///
/// // Basic skill name extraction
/// assert_eq!(extract_skill_name("/commit"), Some("commit".to_string()));
/// assert_eq!(extract_skill_name("/commit -m \"msg\""), Some("commit".to_string()));
///
/// // Quoted skill names (preserves quotes)
/// assert_eq!(extract_skill_name("/\"my skill\" arg1"), Some("\"my skill\"".to_string()));
/// assert_eq!(extract_skill_name("/'single quoted'"), Some("'single quoted'".to_string()));
///
/// // Skill names with special characters
/// assert_eq!(extract_skill_name("/sdd:plan"), Some("sdd:plan".to_string()));
///
/// // Invalid inputs
/// assert_eq!(extract_skill_name(""), None);
/// assert_eq!(extract_skill_name("/"), None);
/// assert_eq!(extract_skill_name("no-slash"), None);
/// ```
#[must_use]
pub fn extract_skill_name(command: &str) -> Option<String> {
    let trimmed = command.trim_start();

    // Must start with '/'
    if !trimmed.starts_with('/') {
        return None;
    }

    // Skip the leading '/'
    let rest = &trimmed[1..];

    // Skip any whitespace after the '/'
    let rest = rest.trim_start();

    if rest.is_empty() {
        return None;
    }

    // Determine if the skill name is quoted or unquoted
    let first_char = rest.chars().next()?;

    if first_char == '"' || first_char == '\'' {
        // Extract quoted skill name (preserving quotes)
        extract_quoted_token(rest, first_char)
    } else {
        // Extract unquoted skill name (terminated by whitespace)
        extract_unquoted_token(rest)
    }
}

/// Extracts a quoted token from the input, preserving the surrounding quotes.
///
/// Handles escape sequences within the quoted string:
/// - `\"` within double quotes becomes a literal `"`
/// - `\'` within single quotes becomes a literal `'`
/// - `\\` becomes a literal `\`
///
/// # Arguments
///
/// * `input` - The input string starting with a quote character
/// * `quote_char` - The quote character (`"` or `'`)
///
/// # Returns
///
/// The quoted token including the surrounding quotes, or `None` if the quote is unclosed.
fn extract_quoted_token(input: &str, quote_char: char) -> Option<String> {
    let mut chars = input.chars().peekable();
    let mut result = String::new();

    // Consume the opening quote
    let opening = chars.next()?;
    debug_assert_eq!(opening, quote_char);
    result.push(opening);

    let mut found_closing = false;

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            // Handle escape sequence
            if let Some(&next_ch) = chars.peek() {
                // Only escape quote chars and backslash
                if next_ch == quote_char || next_ch == '\\' {
                    result.push(chars.next().unwrap());
                } else {
                    // Keep the backslash for other characters
                    result.push(ch);
                }
            } else {
                // Trailing backslash
                result.push(ch);
            }
        } else if ch == quote_char {
            // Found closing quote
            result.push(ch);
            found_closing = true;
            break;
        } else {
            result.push(ch);
        }
    }

    if found_closing {
        Some(result)
    } else {
        // Unclosed quote - return None as per requirements
        None
    }
}

/// Extracts an unquoted token from the input (terminated by whitespace).
///
/// # Arguments
///
/// * `input` - The input string starting with a non-quote character
///
/// # Returns
///
/// The token as a string, or `None` if the input is empty.
fn extract_unquoted_token(input: &str) -> Option<String> {
    if input.is_empty() {
        return None;
    }

    // Take characters until whitespace
    let token: String = input.chars().take_while(|c| !c.is_whitespace()).collect();

    if token.is_empty() {
        None
    } else {
        Some(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Basic functionality tests =====

    #[test]
    fn test_simple_skill_name() {
        assert_eq!(extract_skill_name("/commit"), Some("commit".to_string()));
    }

    #[test]
    fn test_skill_name_with_args() {
        assert_eq!(
            extract_skill_name("/commit -m \"fix: update docs\""),
            Some("commit".to_string())
        );
    }

    #[test]
    fn test_skill_name_with_colon() {
        assert_eq!(
            extract_skill_name("/sdd:plan"),
            Some("sdd:plan".to_string())
        );
    }

    #[test]
    fn test_skill_name_with_hyphen() {
        assert_eq!(
            extract_skill_name("/review-pr 123"),
            Some("review-pr".to_string())
        );
    }

    #[test]
    fn test_skill_name_with_underscore() {
        assert_eq!(
            extract_skill_name("/my_skill arg"),
            Some("my_skill".to_string())
        );
    }

    // ===== Quoted skill names (preserve quotes) =====

    #[test]
    fn test_double_quoted_skill_name() {
        assert_eq!(
            extract_skill_name("/\"my skill\" arg1"),
            Some("\"my skill\"".to_string())
        );
    }

    #[test]
    fn test_single_quoted_skill_name() {
        assert_eq!(
            extract_skill_name("/'my skill' arg1"),
            Some("'my skill'".to_string())
        );
    }

    #[test]
    fn test_quoted_skill_with_special_chars() {
        assert_eq!(
            extract_skill_name("/\"skill:with:colons\" arg"),
            Some("\"skill:with:colons\"".to_string())
        );
    }

    // ===== Escape sequences within quotes =====

    #[test]
    fn test_escaped_double_quote_in_skill_name() {
        // /\"escaped\"quote\" should extract "escaped\"quote"
        assert_eq!(
            extract_skill_name("/\"escaped\\\"quote\""),
            Some("\"escaped\"quote\"".to_string())
        );
    }

    #[test]
    fn test_escaped_single_quote_in_skill_name() {
        assert_eq!(
            extract_skill_name("/'it\\'s a skill'"),
            Some("'it's a skill'".to_string())
        );
    }

    #[test]
    fn test_escaped_backslash_in_skill_name() {
        assert_eq!(
            extract_skill_name("/\"path\\\\to\\\\skill\""),
            Some("\"path\\to\\skill\"".to_string())
        );
    }

    #[test]
    fn test_backslash_before_non_special_char() {
        // Backslash before non-special char should be preserved
        assert_eq!(
            extract_skill_name("/\"skill\\nname\""),
            Some("\"skill\\nname\"".to_string())
        );
    }

    // ===== Empty and invalid input tests =====

    #[test]
    fn test_empty_input() {
        assert_eq!(extract_skill_name(""), None);
    }

    #[test]
    fn test_just_slash() {
        assert_eq!(extract_skill_name("/"), None);
    }

    #[test]
    fn test_slash_with_only_spaces() {
        assert_eq!(extract_skill_name("/   "), None);
    }

    #[test]
    fn test_no_leading_slash() {
        assert_eq!(extract_skill_name("commit -m \"message\""), None);
    }

    #[test]
    fn test_whitespace_before_slash() {
        // Leading whitespace is allowed, but command must still start with /
        assert_eq!(
            extract_skill_name("  /commit arg"),
            Some("commit".to_string())
        );
    }

    // ===== Whitespace handling =====

    #[test]
    fn test_slash_with_space_before_skill() {
        // Space after slash, skill name follows
        assert_eq!(
            extract_skill_name("/  commit arg"),
            Some("commit".to_string())
        );
    }

    #[test]
    fn test_multiple_spaces_after_slash() {
        assert_eq!(
            extract_skill_name("/    spaced-skill"),
            Some("spaced-skill".to_string())
        );
    }

    #[test]
    fn test_tab_after_slash() {
        assert_eq!(extract_skill_name("/\tskill"), Some("skill".to_string()));
    }

    // ===== Unclosed quotes =====

    #[test]
    fn test_unclosed_double_quote() {
        assert_eq!(extract_skill_name("/\"unclosed"), None);
    }

    #[test]
    fn test_unclosed_single_quote() {
        assert_eq!(extract_skill_name("/'unclosed"), None);
    }

    #[test]
    fn test_unclosed_quote_with_content() {
        assert_eq!(extract_skill_name("/\"skill with spaces"), None);
    }

    // ===== Edge cases =====

    #[test]
    fn test_empty_quoted_string() {
        assert_eq!(extract_skill_name("/\"\""), Some("\"\"".to_string()));
    }

    #[test]
    fn test_empty_single_quoted_string() {
        assert_eq!(extract_skill_name("/''"), Some("''".to_string()));
    }

    #[test]
    fn test_skill_name_only_special_chars() {
        assert_eq!(extract_skill_name("/::-::"), Some("::-::".to_string()));
    }

    #[test]
    fn test_unicode_skill_name() {
        assert_eq!(
            extract_skill_name("/\u{1F680}rocket"),
            Some("\u{1F680}rocket".to_string())
        );
    }

    #[test]
    fn test_unicode_in_quoted_skill_name() {
        assert_eq!(
            extract_skill_name("/\"\u{1F680} rocket launch\""),
            Some("\"\u{1F680} rocket launch\"".to_string())
        );
    }

    #[test]
    fn test_nested_quotes_different_types() {
        // Double quotes containing single quotes
        assert_eq!(
            extract_skill_name("/\"it's working\""),
            Some("\"it's working\"".to_string())
        );
    }

    #[test]
    fn test_single_quotes_containing_double() {
        assert_eq!(
            extract_skill_name("/'say \"hello\"'"),
            Some("'say \"hello\"'".to_string())
        );
    }

    #[test]
    fn test_trailing_backslash_in_quoted() {
        // Trailing backslash at end of quoted string before closing quote
        assert_eq!(
            extract_skill_name("/\"skill\\\\\""),
            Some("\"skill\\\"".to_string())
        );
    }

    #[test]
    fn test_trailing_backslash_unclosed() {
        // Unclosed quote ending with backslash
        assert_eq!(extract_skill_name("/\"skill\\"), None);
    }

    // ===== Real-world examples from tasks.md =====

    #[test]
    fn test_example_commit_with_message() {
        assert_eq!(
            extract_skill_name("/commit -m \"fix: update docs\""),
            Some("commit".to_string())
        );
    }

    #[test]
    fn test_example_quoted_my_skill() {
        assert_eq!(
            extract_skill_name("/\"my skill\" arg1"),
            Some("\"my skill\"".to_string())
        );
    }

    #[test]
    fn test_example_sdd_plan() {
        assert_eq!(
            extract_skill_name("/sdd:plan"),
            Some("sdd:plan".to_string())
        );
    }

    // ===== Additional coverage =====

    #[test]
    fn test_skill_name_with_numbers() {
        assert_eq!(
            extract_skill_name("/v2-skill"),
            Some("v2-skill".to_string())
        );
    }

    #[test]
    fn test_skill_name_is_number() {
        assert_eq!(extract_skill_name("/123"), Some("123".to_string()));
    }

    #[test]
    fn test_only_whitespace_input() {
        assert_eq!(extract_skill_name("   "), None);
    }

    #[test]
    fn test_newline_in_input() {
        assert_eq!(
            extract_skill_name("/skill\nmore content"),
            Some("skill".to_string())
        );
    }

    #[test]
    fn test_skill_followed_by_tab() {
        assert_eq!(extract_skill_name("/skill\targ"), Some("skill".to_string()));
    }
}
