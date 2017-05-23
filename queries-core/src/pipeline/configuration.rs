use std::collections::HashMap;

use pipeline::rule_based::RuleBasedParserConfiguration;
use pipeline::probabilistic::ProbabilisticParserConfiguration;

#[derive(Debug, Deserialize)]
pub struct SnipsConfiguration {
    pub model: Model,
    pub entities: HashMap<String, Entity>
}

#[derive(Debug, Deserialize)]
pub struct Model {
    pub rule_based_parser: Option<RuleBasedParserConfiguration>,
    pub probabilistic_parser: Option<ProbabilisticParserConfiguration>,
}

#[derive(Debug, Deserialize)]
pub struct Entity {
    pub automatically_extensible: bool,
    pub utterances: HashMap<String, String>
}

#[cfg(test)]
mod tests {
    use super::SnipsConfiguration;

    use utils;

    #[test]
    fn deserialization_works() {
        let retrieved: SnipsConfiguration = utils::parse_json("tests/nlu_engine_sample.json");
        assert_eq!("en", retrieved.model.rule_based_parser.unwrap().language);
        assert_eq!("en", retrieved.model.probabilistic_parser.unwrap().language_code);
    }
}
