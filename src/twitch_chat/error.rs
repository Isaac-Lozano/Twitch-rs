use std::fmt;
use std::error::Error;
use std::io::Error as IOError;

use std::string::FromUtf8Error;

#[derive(Debug)]
pub enum ClientError<'a>
{
    IOError(IOError),
    InvalidStateError(&'a str),
    ParsingError,
}

impl<'a> fmt::Display for ClientError<'a>
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result
    {
        match *self
        {
            ClientError::IOError(ref e) =>
                write!(fmt, "IOError: {}", e),
            ClientError::InvalidStateError(ref s) =>
                write!(fmt, "InvalidStateError: {}", s),
            ClientError::ParsingError =>
                write!(fmt, "ParsingError"),
        }
    }
}

impl<'a> Error for ClientError<'a>
{
    fn description(&self) -> &str
    {
        match *self
        {
            ClientError::IOError(ref e) =>
                e.description(),
            ClientError::InvalidStateError(ref s) =>
                s,
            ClientError::ParsingError =>
                "Error parsing IRC line",
        }
    }

    fn cause(&self) -> Option<&Error>
    {
        match *self
        {
            ClientError::IOError(ref e) =>
                Some(e),
            ClientError::InvalidStateError(ref s) =>
                None,
            ClientError::ParsingError =>
                None,
        }
    }
}

impl<'a> From<IOError> for ClientError<'a>
{
    fn from(e: IOError) -> ClientError<'a>
    {
        ClientError::IOError(e)
    }
}

impl<'a> From<FromUtf8Error> for ClientError<'a>
{
    fn from(e: FromUtf8Error) -> ClientError<'a>
    {
        /* XXX */
        ClientError::ParsingError
    }
}

pub type ClientResult<'a, T> = Result<T, ClientError<'a>>;
