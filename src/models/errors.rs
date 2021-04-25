use std::fmt;

use crate::convert_error;

#[derive(Clone, Debug)]
pub struct LeagueHelperError {
    pub message: String,
}

impl fmt::Display for LeagueHelperError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl LeagueHelperError {
    pub fn new<S: AsRef<str>>(message: S) -> Self {
        Self {
            message: message.as_ref().to_string(),
        }
    }
}

impl std::error::Error for LeagueHelperError {}

convert_error!(reqwest::Error);
convert_error!(serde_json::Error);
convert_error!(json::Error);
convert_error!(regex::Error);
convert_error!(std::num::ParseIntError);
convert_error!(std::io::Error);

#[macro_export]
macro_rules! convert_error {
    ($err_type:ty) => {
        impl From<$err_type> for LeagueHelperError {
            fn from(err: $err_type) -> Self {
                let err_str = err.to_string();

                LeagueHelperError::new(err_str)
            }
        }
    };

    ($err_type:ty, $custom_message:expr) => {
        impl From<$err_type> for LeagueHelperError {
            fn from(err: $err_type) -> Self {
                let err_str = err.to_string();

                LeagueHelperError::new($custom_message)
            }
        }
    };
}
