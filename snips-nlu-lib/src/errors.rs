error_chain! {
    links {
        SnipsNluOntology(::snips_nlu_ontology::errors::Error, ::snips_nlu_ontology::errors::ErrorKind);
    }

    foreign_links {
        Io(::std::io::Error);
        NdArray(::ndarray::ShapeError);
        Csv(::csv::Error);
        Zip(::zip::result::ZipError);
        Regex(::regex::Error);
        Crfsuite(::crfsuite::Error);
        Base64(::base64::DecodeError);
        Utf8(::std::string::FromUtf8Error);
        PackedResources(::resources_packed::Error);
        SerdeJson(::serde_json::Error);
    }

    errors {
        ConfigLoad(path: String) {
            description("Config file not found")
            display("Unable to read file `{}`", path)
        }

        WrongModelVersion(model_version: String) {
            description("Model version doesn't match")
            display("Given model version {} doesn't match. Expected model version {}", model_version, ::SnipsNluEngine::model_version())
        }
    }
}

impl<T> ::std::convert::From<::std::sync::PoisonError<T>> for Error {
    fn from(pe: ::std::sync::PoisonError<T>) -> Error {
        format!("Poisoning error: {:?}", pe).into()
    }
}
