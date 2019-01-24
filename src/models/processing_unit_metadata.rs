#[derive(Debug, Deserialize, Copy, Clone, PartialEq, Eq)]
#[serde(tag = "unit_name")]
#[serde(rename_all = "snake_case")]
pub enum ProcessingUnitMetadata {
    DeterministicIntentParser,
    ProbabilisticIntentParser,
    CrfSlotFiller,
    LogRegIntentClassifier,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn deserialize_works() {
        let data = r#"{
                        "unit_name": "crf_slot_filler"
                      }"#;
        let metadata: ProcessingUnitMetadata = serde_json::from_str(data).unwrap();
        assert_eq!(ProcessingUnitMetadata::CrfSlotFiller, metadata);
    }
}
