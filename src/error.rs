use std::fmt;
use std::error::Error as StdError;
use std::result::Result as StdResult;

use serde;
use rlua::Error as LuaError;


#[derive(Debug)]
pub struct Error(LuaError);

pub type Result<T> = StdResult<T, Error>;

impl From<LuaError> for Error {
    fn from(err: LuaError) -> Error {
        Error(err)
    }
}

impl From<Error> for LuaError {
    fn from(err: Error) -> LuaError {
        err.0
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(fmt)
    }
}

impl StdError for Error {
    fn description(&self) -> &'static str {
        "Failed to serialize to Lua value"
    }
}

impl serde::ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error(LuaError::ToLuaConversionError {
            from: "serialize",
            to: "value",
            message: Some(format!("{}", msg))
        })
    }
}

impl serde::de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error(LuaError::FromLuaConversionError {
            from: "value",
            to: "deserialize",
            message: Some(format!("{}", msg))
        })
    }
}
