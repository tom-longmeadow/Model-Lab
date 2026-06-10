
/// Returns the indices of the `n` entries with the **lowest** score.
pub(crate) fn least_n(scores: &[f64], n: usize) -> Vec<usize> {
    select_n_by(scores, n, |a, b| a.partial_cmp(b).unwrap())
}

/// Returns the indices of the `n` entries with the **greatest** score.
pub(crate) fn greatest_n(scores: &[f64], n: usize) -> Vec<usize> {
    select_n_by(scores, n, |a, b| b.partial_cmp(a).unwrap())
}

fn select_n_by<F>(scores: &[f64], n: usize, cmp: F) -> Vec<usize>
where
    F: Fn(&f64, &f64) -> std::cmp::Ordering,
{
    let n = n.min(scores.len());
    let mut indices: Vec<usize> = (0..scores.len()).collect();
    indices.select_nth_unstable_by(n.saturating_sub(1), |&a, &b| cmp(&scores[a], &scores[b]));
    indices.truncate(n);
    indices
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn least_n_returns_lowest_score_indices() {
        let scores = vec![5.0, 1.0, 3.0, 0.5, 4.0];
        let mut result = least_n(&scores, 2);
        result.sort_unstable();
        assert_eq!(result, vec![1, 3]); // 0.5 and 1.0
    }

    #[test]
    fn greatest_n_returns_highest_score_indices() {
        let scores = vec![5.0, 1.0, 3.0, 0.5, 4.0];
        let mut result = greatest_n(&scores, 2);
        result.sort_unstable();
        assert_eq!(result, vec![0, 4]); // 5.0 and 4.0
    }

    #[test]
    fn least_n_capped_at_len() {
        let scores = vec![1.0, 2.0];
        assert_eq!(least_n(&scores, 100).len(), 2);
    }
}
 