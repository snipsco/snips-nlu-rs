error_chain! {
    foreign_links {
        Io(::std::io::Error);
        NdArray(::ndarray::ShapeError);
        Csv(::csv::Error);
        Zip(::zip::result::ZipError);
        Fst(::fst::Error);
        Preprocessor(::preprocessing::Error);
        Regex(::regex::Error);
    }
}
