//! Fixed-pitch word wrapping.

/// Wrap `text` to lines of at most `width` characters. Hand-made line breaks (`\n`) are kept,
/// words longer than a line are cut, and leading indentation of a hand-made line is kept when it
/// fits. Never returns an empty list: empty text is one empty line.
pub fn wrap(text: &str, width: usize) -> Vec<String> {
    let width = width.max(1);
    let mut out = Vec::new();
    for hard in text.split('\n') {
        let hard = hard.trim_end();
        if hard.chars().count() <= width {
            out.push(hard.to_owned());
            continue;
        }
        let indent: String = hard.chars().take_while(|c| c.is_whitespace()).map(|_| ' ').collect();
        let mut line = if indent.chars().count() < width / 2 { indent } else { String::new() };
        let mut line_len = line.chars().count();
        let mut has_word = false;
        for word in hard.split_whitespace() {
            let mut word: Vec<char> = word.chars().collect();
            loop {
                let sep = usize::from(has_word);
                if line_len + sep + word.len() <= width {
                    if has_word {
                        line.push(' ');
                    }
                    line.extend(word.iter());
                    line_len += sep + word.len();
                    has_word = true;
                    break;
                }
                if has_word {
                    out.push(std::mem::take(&mut line));
                    line_len = 0;
                    has_word = false;
                    continue;
                }
                if line_len > 0 {
                    // Only indentation so far: drop it rather than leave a blank line.
                    line.clear();
                    line_len = 0;
                    continue;
                }
                // A word longer than the whole line: cut it.
                let rest = word.split_off(width.min(word.len()));
                out.push(word.into_iter().collect());
                word = rest;
                if word.is_empty() {
                    break;
                }
            }
        }
        if has_word {
            out.push(line);
        }
    }
    if out.is_empty() {
        out.push(String::new());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::wrap;

    #[test]
    fn wraps_on_words() {
        assert_eq!(wrap("one two three four", 9), ["one two", "three", "four"]);
    }

    #[test]
    fn keeps_hard_breaks_and_blank_lines() {
        assert_eq!(wrap("a\n\nb", 10), ["a", "", "b"]);
    }

    #[test]
    fn cuts_long_words() {
        assert_eq!(wrap("abcdefghij", 4), ["abcd", "efgh", "ij"]);
    }

    #[test]
    fn empty_is_one_line_and_zero_width_is_safe() {
        assert_eq!(wrap("", 10), [""]);
        assert_eq!(wrap("ab", 0), ["a", "b"]);
    }

    #[test]
    fn multibyte_counts_characters() {
        assert_eq!(wrap("ééé ééé", 3), ["ééé", "ééé"]);
    }
}
