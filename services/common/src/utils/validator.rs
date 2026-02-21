use validator::ValidationError;

/// Validates that a string does not contain null bytes (\0).
/// Works for both String and Option<String> fields (validator unwraps Option).
pub fn validate_no_null_bytes(val: &str) -> Result<(), ValidationError> {
    if val.contains('\0') {
        return Err(ValidationError::new("no_null_bytes"));
    }
    Ok(())
}
