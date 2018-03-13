use std::fmt;
use std::fmt::Display;
use failure::Error;
use failure::Fail;

pub struct PrettyFail<'a>(&'a Fail);

impl<'a> Display for PrettyFail<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        writeln!(fmt, "Error: {}", &self.0)?;

        let mut x: &Fail = self.0;
        while let Some(cause) = x.cause() {
            writeln!(fmt, " -> caused by: {}", &cause)?;
            x = cause;
        }
        if let Some(backtrace) = x.backtrace() {
            writeln!(fmt, "{:?}", backtrace)?;
        }
        Ok(())
    }
}

pub trait ErrorExt {
    fn pretty(&self) -> PrettyFail;
}

impl ErrorExt for Error {
    fn pretty(&self) -> PrettyFail {
        PrettyFail(self.cause())
    }
}
