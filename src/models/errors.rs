use std::fmt;
use std::fmt::Formatter;

use lcu_driver::errors::LcuDriverError;

use crate::convert_error;
use crate::Result;

#[derive(Eq, PartialEq)]
pub enum LeagueHelperError {
    DriverError(LcuDriverError),
    Other(String),
}

pub trait ErrorExt<T, M>
where
    M: AsRef<str>,
{
    fn context(self, msg: M) -> Result<T>;
}

impl<T, M> ErrorExt<T, M> for Option<T>
where
    M: AsRef<str>,
{
    fn context(self, msg: M) -> Result<T> {
        self.ok_or_else(|| LeagueHelperError::Other(msg.as_ref().to_owned()))
    }
}

impl fmt::Display for LeagueHelperError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            LeagueHelperError::DriverError(e) => return e.fmt(f),
            LeagueHelperError::Other(e) => e,
        };

        write!(f, "{}", message)
    }
}

impl fmt::Debug for LeagueHelperError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

impl LeagueHelperError {
    pub fn new<S: AsRef<str>>(message: S) -> Self {
        Self::Other(message.as_ref().to_string())
    }
}

impl std::error::Error for LeagueHelperError {}

impl From<LcuDriverError> for LeagueHelperError {
    fn from(lcu_err: LcuDriverError) -> Self {
        LeagueHelperError::DriverError(lcu_err)
    }
}

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
