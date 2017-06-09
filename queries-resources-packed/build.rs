extern crate phf_codegen;
extern crate queries_resources;

use std::env;
use std::fs;
use std::io::{BufWriter, Write};
use std::path::Path;


fn main() {
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("phf.rs");

    if let Ok(p) = env::var("SNIPS_PHF_OVERRIDE") {
        println!("cargo:warning=Overriding PHF generation with {}", p);
        fs::copy(Path::new(&p), path).unwrap();
    } else {
        let mut file = BufWriter::new(fs::File::create(&path).unwrap());

        macro_rules! stem {
            ($lang:ident) => {
                write!(&mut file,
                       "static STEMS_{}: ::phf::Map<&'static str, &'static str> = ",
                       stringify!($lang).to_uppercase()).unwrap();
                let mut builder = phf_codegen::Map::new();
                let stems = queries_resources::stems::$lang().unwrap();

                for (key, value) in stems.into_iter() {
                    builder.entry(key, &format!("\"{}\"", value));
                }

                builder.build(&mut file).unwrap();

                write!(&mut file, ";\n").unwrap();
            };
        }

        macro_rules! word_clusters {
            ($lang:ident, $cluster_name:ident) => {
                write!(&mut file,
                       "static WORD_CLUSTERS_{}_{}: ::phf::Map<&'static str, &'static str> = ",
                       stringify!($lang).to_uppercase(),
                       stringify!($cluster_name).to_uppercase()).unwrap();
                let mut builder = phf_codegen::Map::new();
                let clusters = queries_resources::word_clusters::$lang::$cluster_name().unwrap();

                for (key, value) in clusters.into_iter() {
                    builder.entry(key, &format!("\"{}\"", value));
                }

                builder.build(&mut file).unwrap();

                write!(&mut file, ";\n").unwrap();
            };
        }


        macro_rules! gazetteer {
            ($lang:ident, $gazetteer_name:ident) => {
                write!(&mut file,
                       "static GAZETTEER_{}_{}: ::phf::Set<&'static str> = ",
                       stringify!($lang).to_uppercase(),
                       stringify!($gazetteer_name).to_uppercase()).unwrap();
                let mut builder = phf_codegen::Set::new();
                let clusters = queries_resources::gazetteer::$lang::$gazetteer_name().unwrap();

                for value in clusters.into_iter() {
                    builder.entry(value);
                }

                builder.build(&mut file).unwrap();

                write!(&mut file, ";\n").unwrap();
            };
        }

        stem!(en);
        stem!(fr);
        stem!(es);

        word_clusters!(en, brown_clusters);

        gazetteer!(en, top_10000_nouns);
        gazetteer!(en, cities_us);
        gazetteer!(en, cities_world);
        gazetteer!(en, countries);
        gazetteer!(en, states_us);
        gazetteer!(en, stop_words);
        gazetteer!(en, street_identifier);
        gazetteer!(en, top_10000_words);

        gazetteer!(en, top_10000_nouns_stem);
        gazetteer!(en, cities_us_stem);
        gazetteer!(en, cities_world_stem);
        gazetteer!(en, countries_stem);
        gazetteer!(en, states_us_stem);
        gazetteer!(en, stop_words_stem);
        gazetteer!(en, street_identifier_stem);
        gazetteer!(en, top_10000_words_stem);
    }
    // we generate some files based on dependencies of this build script and not files in this
    // project, so we can deactivate the auto rebuild on each file change
    println!("cargo:rerun-if-changed=build.rs")
}
