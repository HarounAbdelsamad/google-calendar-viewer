use std::fmt;

/// Errors returned to the frontend. Serialized as a plain string; `AuthExpired`
/// is the sentinel "AUTH_EXPIRED" so the UI knows to show the sign-in screen.
#[derive(Debug)]
pub enum Error {
    AuthExpired,
    Config,
    Http(String),
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::AuthExpired => write!(f, "AUTH_EXPIRED"),
            Error::Config => write!(
                f,
                "Missing GOOGLE_CLIENT_ID / GOOGLE_CLIENT_SECRET. Copy .env.example to .env, fill it in, and rebuild."
            ),
            Error::Http(msg) => write!(f, "Network error: {msg}"),
            Error::Other(msg) => write!(f, "{msg}"),
        }
    }
}

impl serde::Serialize for Error {
    fn serialize<S: serde::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::Http(e.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Other(e.to_string())
    }
}

impl From<keyring::Error> for Error {
    fn from(e: keyring::Error) -> Self {
        Error::Other(format!("Credential store error: {e}"))
    }
}
