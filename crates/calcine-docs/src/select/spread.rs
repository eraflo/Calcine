//! The order in which to skim a document.

/// Passage indexes of a document of `length` passages, in the order to add
/// them when skimming: the start, then ever finer splits of the rest.
pub(super) fn spread(length: usize) -> Vec<usize> {
    let mut order = Vec::with_capacity(length);
    let mut seen = vec![false; length];
    let mut parts = 1;
    while order.len() < length {
        for part in 0..parts {
            let at = part * length / parts;
            if at < length && !seen[at] {
                seen[at] = true;
                order.push(at);
            }
        }
        if parts >= length {
            // Whatever the splits missed.
            order.extend((0..length).filter(|&at| !seen[at]));
            break;
        }
        parts *= 2;
    }
    order
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spread_starts_at_the_beginning_and_covers_everything() {
        assert_eq!(spread(4), vec![0, 2, 1, 3]);
        assert_eq!(spread(1), vec![0]);
        assert_eq!(spread(0), Vec::<usize>::new());
        let mut all = spread(13);
        all.sort_unstable();
        assert_eq!(all, (0..13).collect::<Vec<_>>());
    }
}
