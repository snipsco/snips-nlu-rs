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
        let tokens_probabilities =
            arr2(&[[0.1, 1.4, 0.3], [0.6, 0.5, 0.4], [0.7, 0.8, 0.9], [1.2, 1.1, 1.0]]);

        let expected = vec!["me restaurant", "book", "a"];
        let slots = compute_slots(&tokens, &tokens_probabilities);

        assert_eq!(slots, expected);
    }
}
