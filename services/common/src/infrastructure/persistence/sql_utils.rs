/// Escape special characters in LIKE patterns
/// Prevents SQL injection through wildcards
pub fn escape_like_pattern(s: &str) -> String {
    s.replace('\\', r"\\")
        .replace('%', r"\%")
        .replace('_', r"\_")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_like_pattern() {
        assert_eq!(escape_like_pattern("test"), "test");
        assert_eq!(escape_like_pattern("test%"), r"test\%");
        assert_eq!(escape_like_pattern("test_user"), r"test\_user");
        assert_eq!(
            escape_like_pattern("test\\user_profile"),
            r"test\\user\_profile"
        );
        assert_eq!(escape_like_pattern("%_\\"), r"\%\_\\");
    }
}
