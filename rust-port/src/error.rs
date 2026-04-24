use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrxError {
    pub message: String,
}

impl CrxError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl From<&str> for CrxError {
    fn from(msg: &str) -> Self {
        Self::new(msg)
    }
}

impl From<String> for CrxError {
    fn from(msg: String) -> Self {
        Self::new(msg)
    }
}

impl Display for CrxError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for CrxError {}