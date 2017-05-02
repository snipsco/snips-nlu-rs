error_chain! {
    foreign_links {
        Io(::std::io::Error);
        NdArray(::ndarray::ShapeError);
        Protobuf(::protobuf::ProtobufError);
        Csv(::csv::Error);
        Zip(::zip::result::ZipError);
        Fst(::fst::Error);
        Preprocessor(::preprocessing::Error);
    }
}

impl ::std::convert::From<::tensorflow::Status> for Error {
    fn from(tfs: ::tensorflow::Status) -> Error {
        format!("Tensorflow error: {:?}", tfs).into()
    }
}
