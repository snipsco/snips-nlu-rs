use failure::Fail;

#[derive(Debug, Fail)]
pub enum SnipsNluError {
    #[fail(display = "Unable to read file '{}'", _0)]
    ModelLoad(String),
    #[fail(display = "Mismatched model version: model is {} but runner is {}", model, runner)]
    WrongModelVersion{ model: String, runner: &'static str},
    #[fail(display = "Unknown intent: '{}'", _0)]
    UnknownIntent(String),
    #[fail(display = "Internal error: {}", _0)]
    InternalError(String),
}

pub type Result<T> = ::std::result::Result<T, ::failure::Error>;
