pub mod creator;
pub mod deletor;
pub mod lifecycle;

/// Returns the indices of the `n` entries with the **lowest** score.
pub(super) fn least_n(scores: &[f64], n: usize) -> Vec<usize> {
    select_n_by(scores, n, |a, b| a.partial_cmp(b).unwrap())
}

/// Returns the indices of the `n` entries with the **greatest** score.
pub(super) fn greatest_n(scores: &[f64], n: usize) -> Vec<usize> {
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