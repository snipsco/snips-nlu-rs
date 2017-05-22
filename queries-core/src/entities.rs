use rustling_ontology::{Lang, Parser, DimensionKind, build_parser, ParsingContext, Output};
use std::ops::Range;
use errors::*;

pub struct RustlingParser {
    parser: Parser,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct RustlingEntity {
    pub value: String,
    pub range: Range<usize>,// Byte range
    pub kind: EntityKind,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub enum EntityKind {
    Time,
    Duration,
    Number,
}

impl RustlingParser {
    pub fn new(lang: Lang) -> Result<RustlingParser> {
        Ok(RustlingParser {
            parser: build_parser(lang)?
        })
    }

    pub fn extract_entities(&self, sentence: &str) -> Result<Vec<RustlingEntity>> {
        let context = ParsingContext::default();
        let kind_order = vec![DimensionKind::Number, DimensionKind::Time, DimensionKind::Duration];
        let mut entities = self.parser.parse_with_kind_order(sentence, &context, &kind_order)?
            .iter()
            .filter_map(|m| {
                EntityKind::from_rustling_output(&m.value)
                    .map(|e| {
                        RustlingEntity {
                            value: sentence[m.byte_range.0..m.byte_range.1].into(),
                            range: Range { start: m.byte_range.0, end: m.byte_range.1 },
                            kind: e
                        }
                    })
            })
            .collect::<Vec<_>>();
        entities.sort_by_key(|e| e.range.start);
        Ok(entities)

    }
}

impl EntityKind {

    fn identifier(&self) -> String {
        match self {
            EntityKind::Time => "snips/datetime",
            EntityKind::Number => "snips/number",
            EntityKind::Duration => "snips/duration",
        }
    }

    fn from_rustling_output(v: &Output) -> Option<EntityKind> {
        match v {
            &Output::Time(_) => Some(EntityKind::Time),
            &Output::TimeInterval(_) => Some(EntityKind::Time),
            &Output::Integer(_) => Some(EntityKind::Number),
            &Output::Float(_) => Some(EntityKind::Number),
            &Output::Duration(_) => Some(EntityKind::Duration),
            _ => None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_entities_extraction() {
        let parser = RustlingParser::new(Lang::EN).unwrap();
        assert_eq!(vec![
                    RustlingEntity { value: "two".into(), range: Range { start: 23, end: 26 }, kind: EntityKind::Number },
                    RustlingEntity { value: "tomorrow".into(), range: Range { start: 34, end: 42 }, kind: EntityKind::Time },
                ], parser.extract_entities("Book me restaurant for two people tomorrow").unwrap());

        assert_eq!(vec![
                    RustlingEntity { value: "two weeks".into(), range: Range { start: 19, end: 28 }, kind: EntityKind::Duration },
                ], parser.extract_entities("The weather during two weeks").unwrap());
    }
}
