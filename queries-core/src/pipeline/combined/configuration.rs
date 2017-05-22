use pipeline::rule_based::RuleBasedParserConfiguration;
use pipeline::probabilistic::ProbabilisticParserConfiguration;

#[derive(Debug, Deserialize)]
pub struct SnipsConfiguration {
    pub model: Model,
}

#[derive(Debug, Deserialize)]
pub struct Model {
    pub rule_based_parser: RuleBasedParserConfiguration,
    pub probabilistic_parser: ProbabilisticParserConfiguration,
}

#[cfg(test)]
mod tests {
    use super::SnipsConfiguration;

    use utils;

    #[test]
    fn deserialization_works() {
        let retrieved: SnipsConfiguration = utils::parse_json("tests/nlu_engine_sample.json");
        println!("{:?}", retrieved);
    }
}
