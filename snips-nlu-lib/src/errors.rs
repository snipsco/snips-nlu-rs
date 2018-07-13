#[derive(Debug, Fail)]
pub enum SnipsNluError {
    #[fail(display = "Unable to read file '{}'", _0)]
    ModelLoad(String),
    #[fail(display = "Expected model version {} but found {}", _1, _0)]
    WrongModelVersion(String, &'static str),
}

pub type Result<T> = ::std::result::Result<T, ::failure::Error>;
