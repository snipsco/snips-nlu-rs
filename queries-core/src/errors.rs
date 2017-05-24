error_chain! {
    foreign_links {
        Io(::std::io::Error);
        NdArray(::ndarray::ShapeError);
        Csv(::csv::Error);
        Zip(::zip::result::ZipError);
        Fst(::fst::Error);
        Preprocessor(::preprocessing::Error);
        Regex(::regex::Error);
        Crfsuite(::crfsuite::Error);
        Base64(::base64::DecodeError);
    }
}

impl<T> ::std::convert::From<::std::sync::PoisonError<T>> for Error {
    fn from(pe: ::std::sync::PoisonError<T>) -> Error {
        format!("Poisoning error: {:?}", pe).into()
    }
}
