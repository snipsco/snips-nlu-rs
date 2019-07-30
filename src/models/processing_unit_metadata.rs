use serde::Deserialize;

#[derive(Debug, Deserialize, Copy, Clone, PartialEq, Eq)]
#[serde(tag = "unit_name")]
#[serde(rename_all = "snake_case")]
pub enum ProcessingUnitMetadata {
    DeterministicIntentParser,
    LookupIntentParser,
    ProbabilisticIntentParser,
    CrfSlotFiller,
    LogRegIntentClassifier,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize() {
        let data = r#"{
                        "unit_name": "crf_slot_filler"
                      }"#;
        let metadata: ProcessingUnitMetadata = serde_json::from_str(data).unwrap();
        assert_eq!(ProcessingUnitMetadata::CrfSlotFiller, metadata);
    }
}
