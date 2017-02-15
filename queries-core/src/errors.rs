error_chain! {
    foreign_links {
        Io(::std::io::Error);
        NdArray(::ndarray::ShapeError);
        Serde(::serde_json::Error);
        Protobuf(::protobuf::ProtobufError); 
    }
}
