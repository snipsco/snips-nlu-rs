use ndarray::prelude::*;

use pipeline::SlotValue;

use preprocessing::PreprocessorResult;

pub fn compute_slots(preprocessor_result: &PreprocessorResult,
                     num_slots: usize,
                     tokens_predictions: &Array1<usize>)
                     -> Vec<Vec<SlotValue>> {
    let mut result: Vec<Vec<SlotValue>> = vec![vec![]; num_slots];

    for (i, token) in preprocessor_result.tokens.iter().enumerate() {
        if tokens_predictions[i] == 0 { continue }

        let ref mut tokens = result[tokens_predictions[i] - 1];

        if tokens.is_empty() || (i > 0 && tokens_predictions[i] != tokens_predictions[i - 1]) {
            tokens.push(SlotValue { value: token.value.to_string(), range: token.char_range.clone() });
        } else {
            let existing_token = tokens.last_mut().unwrap(); // checked
            let ref mut existing_token_value = existing_token.value;
            let ref mut existing_token_range = existing_token.range;
            existing_token_value.push_str(&format!(" {}", &token.value));
            existing_token_range.end = token.char_range.end;
        }
    }
    result
}

#[cfg(test)]
mod test {
    use std::ops::Range;

    use ndarray::prelude::*;

    use preprocessing::preprocess;
    use super::SlotValue;
    use super::compute_slots;

    #[test]
    fn slot_filler_works() {
        let text = "Book me a table for tomorrow at Chartier in the evening";
        let entities = r#"[{"end_index":28,"value":"tomorrow","start_index":20,"entity":"%TIME%"},{"end_index": 55,"value":"in the evening","start_index":41,"entity":"%TIME_INTERVAL%"}]"#;
        let preprocess_result = preprocess(text, entities).unwrap();
        let tokens_predictions: Array1<usize> = arr1(&[0, 0, 0, 0, 2, 2, 0, 3, 0, 2, 2]);

        let expected = vec![
            vec![],
            vec![SlotValue { value: "for tomorrow".to_string(), range: Range { start: 16, end: 28 } }],
            vec![SlotValue { value: "Chartier".to_string(), range: Range { start: 32, end: 40 } }],
        ];
        let slots = compute_slots(&preprocess_result, expected.len(), &tokens_predictions);
        println!("slots: {:?}", slots);
        assert_eq!(slots, expected);
    }
}
