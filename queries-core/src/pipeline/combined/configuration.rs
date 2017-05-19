use pipeline::rule_based::configuration::RuleBasedParserConfiguration;
use pipeline::probabilistic::configuration::ProbabilisticParserConfiguration;

#[derive(Debug, Deserialize)]
pub struct NLUConfiguration {
    pub model: Model,
}

#[derive(Debug, Deserialize)]
pub struct Model {
    pub rule_based_parser: RuleBasedParserConfiguration,
    pub probabilistic_parser: ProbabilisticParserConfiguration,
}

#[cfg(test)]
mod tests {
    use super::NLUConfiguration;

    use utils;

    #[test]
    fn deserialization_works() {
        let retrieved: NLUConfiguration = utils::parse_json("tests/nlu_engine_sample.json");
        println!("{:?}", retrieved);
    }
}
