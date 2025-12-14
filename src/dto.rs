use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt::Debug;
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
        return Err(
            ValidationError::new("not_blank").with_message(Cow::from("value cannot be blank"))
        );
    }
    Ok(())
}

#[derive(Deserialize, Debug)]
pub struct RegisterDto {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct RegisterResponse {
    pub token: String,
    pub redirect_url: String,
}

#[derive(Serialize)]
pub struct RoomResponse {
    pub room: String,
}

#[derive(Serialize)]
pub struct Session {
    pub username: String,
}
