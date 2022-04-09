use std::fmt;

/// Wraps several types of errors.
#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
}

/// Defines error kind.
#[derive(Debug, PartialEq)]
pub enum ErrorKind {
    FeatureStateNotFound,
    DuplicateFeatureState,
}

impl Error {
    pub fn new(kind: ErrorKind) -> Error {
        Error { kind }
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            ErrorKind::FeatureStateNotFound => write!(f, "Feature State Not Found"),
            ErrorKind::DuplicateFeatureState => write!(f, "Feature State already exists"),
        }
    }
}
