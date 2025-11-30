use std::borrow::Cow;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

#[derive(Validate, Serialize, Deserialize, Debug)]
pub struct MessageDto {
    #[validate(custom(function = "not_blank"))]
    pub username: String,
    #[validate(custom(function = "not_blank"))]
    pub message: String,
}

pub fn not_blank(value: &str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        return Err(ValidationError::new("not_blank").with_message(Cow::from("value cannot be blank")));
    }
    Ok(())
}