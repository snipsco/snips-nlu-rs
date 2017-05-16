use std::ops::Range;
use std::collections::HashMap;

use pipeline::SlotValue;
use preprocessing::Token;
use utils::convert_char_range;

const BEGINNING_PREFIX: &str = "B-";
const INSIDE_PREFIX: &str = "I-";
const LAST_PREFIX: &str = "L-";
const UNIT_PREFIX: &str = "U-";
const OUTSIDE: &str = "O";

pub enum TaggingScheme {
    IO,
    BIO,
    BILOU,
}

fn tag_name_to_slot_name(tag: &str) -> &str {
    &tag[2..]
}

fn is_start_of_io_slot(tags: &[&str], i: usize) -> bool {
    if i == 0 {
        tags[i] != OUTSIDE
    } else if tags[i] == OUTSIDE {
        false
    } else {
        tags[i - 1] == OUTSIDE
    }
}

fn is_end_of_io_slot(tags: &[&str], i: usize) -> bool {
    if i + 1 == tags.len() {
        tags[i] != OUTSIDE
    } else if tags[i] == OUTSIDE {
        false
    } else {
        tags[i + 1] == OUTSIDE
    }
}

fn is_start_of_bio_slot(tags: &[&str], i: usize) -> bool {
    if i == 0 {
        tags[i] != OUTSIDE
    } else if tags[i] == OUTSIDE {
        false
    } else if tags[i].starts_with(BEGINNING_PREFIX) {
        true
    } else if tags[i - 1] != OUTSIDE {
        false
    } else {
        true
    }
}

fn is_end_of_bio_slot(tags: &[&str], i: usize) -> bool {
    if i + 1 == tags.len() {
        tags[i] != OUTSIDE
    } else if tags[i] == OUTSIDE {
        false
    } else if tags[i + 1].starts_with(INSIDE_PREFIX) {
        false
    } else {
        true
    }
}

fn is_start_of_bilou_slot(tags: &[&str], i: usize) -> bool {
    if i == 0 {
        tags[i] != OUTSIDE
    } else if tags[i] == OUTSIDE {
        false
    } else if tags[i].starts_with(BEGINNING_PREFIX) {
        true
    } else if tags[i].starts_with(UNIT_PREFIX) {
        true
    } else if tags[i - 1].starts_with(UNIT_PREFIX) {
        true
    } else if tags[i - 1].starts_with(LAST_PREFIX) {
        true
    } else if tags[i - 1] != OUTSIDE {
        false
    } else {
        true
    }
}

fn is_end_of_bilou_slot(tags: &[&str], i: usize) -> bool {
    if i + 1 == tags.len() {
        tags[i] != OUTSIDE
    } else if tags[i] == OUTSIDE {
        false
    } else if tags[i + 1] == OUTSIDE {
        true
    } else if tags[i].starts_with(LAST_PREFIX) {
        true
    } else if tags[i].starts_with(UNIT_PREFIX) {
        true
    } else if tags[i + 1].starts_with(BEGINNING_PREFIX) {
        true
    } else if tags[i + 1].starts_with(UNIT_PREFIX) {
        true
    } else {
        false
    }
}

struct SlotRange {
    slot_name: String,
    range: Range<usize>,
}

fn _tags_to_slots<F1, F2>(tags: &[&str],
                          tokens: &[Token],
                          is_start_of_slot: F1,
                          is_end_of_slot: F2)
                          -> Vec<SlotRange>
    where F1: Fn(&[&str], usize) -> bool,
          F2: Fn(&[&str], usize) -> bool
{
    let mut slots: Vec<SlotRange> = Vec::with_capacity(tags.len());

    let mut current_slot_start = 0;
    for (i, tag) in tags.iter().enumerate() {
        if is_start_of_slot(tags, i) {
            current_slot_start = i;
        }
        if is_end_of_slot(tags, i) {
            slots.push(SlotRange {
                           range: tokens[current_slot_start].range.start..tokens[i].range.end,
                           slot_name: tag_name_to_slot_name(tag).to_string(),
                       });
            current_slot_start = i;
        }
    }
    slots
}

fn tags_to_slots(text: &str,
                 tokens: &[Token],
                 tags: &[&str],
                 tagging_scheme: TaggingScheme,
                 intent_slots_mapping: &HashMap<String, String>)
                 -> Vec<(String, SlotValue)> {
    let slots = match tagging_scheme {
        TaggingScheme::IO => _tags_to_slots(tags, tokens, is_start_of_io_slot, is_end_of_io_slot),
        TaggingScheme::BIO => _tags_to_slots(tags, tokens, is_start_of_bio_slot, is_end_of_bio_slot),
        TaggingScheme::BILOU => _tags_to_slots(tags, tokens, is_start_of_bilou_slot, is_end_of_bilou_slot),
    };

    slots
        .into_iter()
        .map(|s| {
            let slot_value = SlotValue {
                range: convert_char_range(text, &s.range),
                value: text[s.range].to_string(),
                entity: intent_slots_mapping[&s.slot_name].to_string(),
            };

            (s.slot_name, slot_value)
        })
        .collect()
}

fn positive_tagging(tagging_scheme: TaggingScheme, slot_name: &str, slot_size: usize) -> Vec<String> {
    match tagging_scheme {
        TaggingScheme::IO => {
            vec![format!("{}{}", INSIDE_PREFIX, slot_name); slot_size]
        },
        TaggingScheme::BIO => {
            if slot_size > 0 {
                let mut v1 = vec![format!("{}{}", BEGINNING_PREFIX, slot_name)];
                let mut v2 = vec![format!("{}{}", INSIDE_PREFIX, slot_name); slot_size - 1];
                v1.append(&mut v2);
                v1
            } else {
                vec![]
            }
        },
        TaggingScheme::BILOU => {
            match slot_size {
                0 => vec![],
                1 => vec![format!("{}{}", UNIT_PREFIX, slot_name)],
                _ => {
                    let mut v1 = vec![format!("{}{}", BEGINNING_PREFIX, slot_name)];
                    let mut v2 = vec![format!("{}{}", INSIDE_PREFIX, slot_name); slot_size - 2];
                    v1.append(&mut v2);
                    v1.push(format!("{}{}", LAST_PREFIX, slot_name));
                    v1
                }
            }
        }
    }
}

fn negative_taggin(size: usize) -> Vec<String> {
    vec![OUTSIDE.to_string(); size]
}

//fn utterance_to_sample(query_data: ?, tagging_scheme: TaggingScheme) -> HashMap<String, Vec<String>> {
    //unimplemented!()
//}

