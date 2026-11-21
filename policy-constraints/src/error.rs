use std::error::Error;
use std::fmt::{self, Display, Formatter};

/// A policy or term that the constraint model cannot represent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelError {
    message: String,
}

impl ModelError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// Prefixes the message with where the error happened.
    pub(crate) fn context(self, context: impl Display) -> Self {
        Self {
            message: format!("{context}: {}", self.message),
        }
    }
}

impl Display for ModelError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for ModelError {}
