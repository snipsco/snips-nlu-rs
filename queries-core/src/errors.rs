error_chain! {
    foreign_links {
        Io(::std::io::Error);
        NdArray(::ndarray::ShapeError);
        Serde(::serde_json::Error);
        Protobuf(::protobuf::ProtobufError);
    }
}

impl ::std::convert::From<::tensorflow::Status> for Error {
    fn from(tfs: ::tensorflow::Status) -> Error {
        format!("Tensorflow error: {:?}", tfs).into()
    }
}
