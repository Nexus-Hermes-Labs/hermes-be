use validator::ValidationError;

/// Validates that a string does not contain null bytes (\0).
/// Works for both String and Option<String> fields (validator unwraps Option).
pub fn validate_no_null_bytes(val: &str) -> Result<(), ValidationError> {
    if val.contains('\0') {
        return Err(ValidationError::new("no_null_bytes"));
    }
    Ok(())
}

/// Validates that a string does not contain C0 (U+0001–U+001F) or C1 (U+0080–U+009F)
/// control characters.
/// Null byte is excluded because `validate_no_null_bytes` covers it.
pub fn validate_no_control_chars(val: &str) -> Result<(), ValidationError> {
    for ch in val.chars() {
        let cp = ch as u32;
        if (1..=0x1F).contains(&cp) || (0x80..=0x9F).contains(&cp) {
            return Err(ValidationError::new("no_control_chars"));
        }
    }
    Ok(())
}
