/// Utility to parse comma-separated string value line.
///
/// - Trims `"` from both ends for the input
/// - For each item, trims whitespace from both ends
pub fn split_csv_line(input: &str) -> Vec<String> {
    input
        .trim_matches('"')
        .split(',')
        .filter_map(|s| {
            let s = s.trim().to_string();
            if s.is_empty() {
                None
            } else {
                Some(s)
            }
        })
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abc_csv() {
        // should ignore whitespaces and `"` at both ends, and ignore empty items
        let input = "\"a,    b , c ,,  \"";
        let expected = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        assert_eq!(split_csv_line(input), expected);
    }

    #[test]
    fn test_empty_csv() {
        assert!(split_csv_line(Default::default()).is_empty());
    }
}
