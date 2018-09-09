use core_graphics::base;
use std::error;
use std::fmt;
use std::result;

#[derive(Debug)]
pub struct CGError {
    error: base::CGError,
}

impl error::Error for CGError {
    fn description(&self) -> &str {
        "a CG error"
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

impl fmt::Display for CGError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CGError: {}", self)
    }
}

impl From<base::CGError> for CGError {
    fn from(e: base::CGError) -> Self {
        CGError { error: e }
    }
}

// Create the Error, ErrorKind, ResultExt, and Result types
error_chain!{
    foreign_links {
        CgError(CGError);
    }
}

pub fn convert_result<T>(result: result::Result<T, base::CGError>) -> result::Result<T, CGError> {
    result.map_err(|e: base::CGError| CGError { error: e })
}
