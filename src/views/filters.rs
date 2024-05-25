use std::{borrow::Borrow, fmt::Display};

/// This filter is used to conditionally set classes in templates like so
/// ```html
/// <a class="btn btn-primary {{ "disabled"|only(is_disabled) }}" >Button</a>
/// ```
pub fn only(s: impl Display, condition: impl Borrow<bool>) -> ::askama::Result<String> {
    if *condition.borrow() {
        Ok(s.to_string())
    } else {
        Ok("".to_string())
    }
}
