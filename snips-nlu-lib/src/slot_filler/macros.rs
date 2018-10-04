#[macro_export]
macro_rules! get_features {
    ([$(($feature_type:ident,$feature_name:ident)),*]) => {
        #[derive(Debug, Copy, Clone, PartialEq, Hash, Eq)]
        pub enum FeatureKind {
            $( $feature_type ),*
        }

        impl FeatureKind {
            pub fn identifier(&self) -> &'static str {
                match self {
                    $(
                        &FeatureKind::$feature_type => stringify!($feature_name),
                    )*
                }
            }
        }

        $(
            impl FeatureKindRepr for $feature_type {
                fn feature_kind(&self) -> FeatureKind {
                    FeatureKind::$feature_type
                }
            }
        )*

        fn get_features(
            f: &FeatureFactory,
            shared_resources: Arc<SharedResources>,
        ) -> Result<Vec<FeatureOffsetter>> {
            let features = match f.factory_name.as_ref() {
                $(
                    stringify!($feature_name) => $feature_type::build_features(&f.args, shared_resources),
                )*
                _ => bail!("Feature {} not implemented", f.factory_name),
            };
            Ok(features?
                .into_iter()
                .map(|feature| FeatureOffsetter { feature, offsets: f.offsets.clone() })
                .collect())
        }
    }
}
