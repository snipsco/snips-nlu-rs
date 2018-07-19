#[derive(Debug, Fail)]
pub enum SnipsNluError {
    #[fail(display = "Unable to read file '{}'", _0)]
    ModelLoad(String),
    #[fail(display = "Expected model version {} but found {}", _1, _0)]
    WrongModelVersion(String, &'static str),
}

#[derive(Debug, Fail)]
pub enum BuiltinEntityParserError {
    #[fail(display = "Unable to load builtin entity parser")]
    LoadingError,
    #[fail(display = "No builtin entity parser loaded for language '{}'", _0)]
    ParserNotLoaded(String),
}

pub type Result<T> = ::std::result::Result<T, ::failure::Error>;
