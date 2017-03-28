use ndarray::prelude::*;

pub fn compute_slots(tokens: &[&str],
                     num_slots: usize,
                     token_probabilities: &Array1<usize>)
                     -> Vec<String> {
    let mut data: Vec<Vec<String>> = vec![vec![]; num_slots];

    for (i, token) in tokens.iter().enumerate() {
        if token_probabilities[i] != 0 {
            data[token_probabilities[i] - 1].push(token.to_string())
        }
    }

    data.iter().map(|tokens| tokens.join(" ")).collect()
}

#[cfg(test)]
mod test {
    use ndarray::prelude::*;
    use pipeline::slot_filler::compute_slots;

    #[test]
    fn slot_filler_works() {
        let tokens = vec!["book", "me", "a", "restaurant"];
        let tokens_predictions: Array1<usize> = arr1(&[1, 1, 3, 0]);

        let expected = vec!["book me", "", "a"];
        let slots = compute_slots(&tokens, expected.len(), &tokens_predictions);

        assert_eq!(slots, expected);
    }
}
