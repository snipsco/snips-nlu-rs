use ndarray::prelude::*;
use rulinalg::utils::argmax;

use pipeline::Probability;

pub fn compute_slots(tokens: &[&str], token_probabilities: &Array2<Probability>) -> Vec<String> {
    let mut data: Vec<Vec<String>> = vec![vec![]; token_probabilities.rows()];

    for col in 0..token_probabilities.cols() {
        let probabilities = token_probabilities.column(col);
        let (max_probability_index, _) = argmax(&probabilities.to_vec());

        let token = tokens[col].to_string();
        data[max_probability_index].push(token);
    }

    data.iter().map(|tokens| tokens.join(" ")).collect()
}

#[cfg(test)]
mod test {
    use ndarray::prelude::*;
    use pipeline::slot_filler::compute_slots;
    use pipeline::Probability;

    #[test]
    fn slot_filler_works() {
        let tokens: Vec<&str> = vec!["Book", "me", "a", "restaurant"];
        let tokens_probabilities: Array2<Probability> = arr2(&[[0.1, 1.4, 0.3],
                                                               [0.6, 0.5, 0.4],
                                                               [0.7, 0.8, 0.9],
                                                               [1.2, 1.1, 1.0]]);

        let expected = vec!["me", "", "", "Book a"];
        let slots = compute_slots(&tokens, &tokens_probabilities);

        assert_eq!(slots, expected);
    }
}
