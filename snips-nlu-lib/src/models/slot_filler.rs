use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct SlotFillerModel {
    pub language_code: String,
    pub intent: String,
    pub slot_name_mapping: HashMap<String, String>,
    pub crf_model_file: String,
    pub config: SlotFillerConfiguration,
}

#[derive(Debug, Deserialize)]
pub struct SlotFillerConfiguration {
    pub tagging_scheme: u8,
    pub feature_factory_configs: Vec<FeatureFactory>,
}

#[derive(Debug, Deserialize)]
pub struct FeatureFactory {
    pub factory_name: String,
    pub offsets: Vec<i32>,
    pub args: HashMap<String, ::serde_json::Value>,
}
