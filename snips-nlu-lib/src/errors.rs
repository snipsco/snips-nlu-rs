#[derive(Debug, Fail)]
pub enum SnipsNluError {
    #[fail(display = "Unable to read file '{}'", _0)]
    ConfigLoad(String),
    #[fail(display = "Given model version {} doesn't match. Expected model version {}", _0, _1)]
    WrongModelVersion(String, &'static str),
}

pub type Result<T> = ::std::result::Result<T, ::failure::Error>;
