use failure::{Backtrace, Context, Fail};
use std::fmt;
use std::fmt::Display;

#[derive(Debug)]
pub struct NluInjectionError {
    inner: Context<NluInjectionErrorKind>,
}

#[derive(Debug, Fail)]
pub enum NluInjectionErrorKind {
    #[fail(display = "Entity is not injectable: {}", msg)]
    EntityNotInjectable { msg: String },
    #[fail(display = "Internal injection error: {}", msg)]
    InternalInjectionError { msg: String },
}

//  Boilerplate
impl Fail for NluInjectionError {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for NluInjectionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl From<NluInjectionErrorKind> for NluInjectionError {
    fn from(kind: NluInjectionErrorKind) -> NluInjectionError {
        NluInjectionError {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<NluInjectionErrorKind>> for NluInjectionError {
    fn from(inner: Context<NluInjectionErrorKind>) -> NluInjectionError {
        NluInjectionError { inner }
    }
}
