//! A reader's own filter over their own feed.
//!
//! This is a view and never a verdict. Nothing here touches reputation, and no
//! list travels the network: whoever runs a client picks their own words, and
//! two clients may screen differently without the protocol having an opinion.
//! That boundary is the whole point — the moment a list decides what an
//! account is worth, whoever writes the list owns the network's speech.
//!
//! What it is good at is exactly what a list is good at: a finite, well-known
//! vocabulary in one language, where a false positive costs a reader nothing
//! but a hidden line, and where writing around the filter is itself a signal.

use std::collections::BTreeMap;

/// Aho-Corasick over folded text: one pass for the whole vocabulary.
#[derive(Clone, Debug, Default)]
pub struct Screen {
    next: Vec<BTreeMap<char, usize>>,
    fail: Vec<usize>,
    hit: Vec<Option<usize>>,
    terms: Vec<String>,
}

impl Screen {
    /// Build a matcher for `terms`. Terms are folded the same way text is.
    #[must_use]
    pub fn build(terms: &[&str]) -> Self {
        let mut screen = Self {
            next: vec![BTreeMap::new()],
            fail: vec![0],
            hit: vec![None],
            terms: Vec::new(),
        };
        for term in terms {
            screen.insert(&fold(term));
        }
        screen.link();
        screen
    }

    fn insert(&mut self, term: &str) {
        if term.is_empty() {
            return;
        }
        let mut node = 0;
        for ch in term.chars() {
            let len = self.next.len();
            let step = *self.next[node].entry(ch).or_insert(len);
            if step == len {
                self.next.push(BTreeMap::new());
                self.fail.push(0);
                self.hit.push(None);
            }
            node = step;
        }
        self.terms.push(term.to_owned());
        self.hit[node] = Some(self.terms.len() - 1);
    }

    /// Breadth-first failure links, the step that makes matching one pass.
    fn link(&mut self) {
        let mut queue: Vec<usize> = self.next[0].values().copied().collect();
        let mut head = 0;
        while head < queue.len() {
            let node = queue[head];
            head += 1;
            let edges: Vec<(char, usize)> =
                self.next[node].iter().map(|(ch, to)| (*ch, *to)).collect();
            for (ch, to) in edges {
                let mut back = self.fail[node];
                while back != 0 && !self.next[back].contains_key(&ch) {
                    back = self.fail[back];
                }
                self.fail[to] = self.next[back]
                    .get(&ch)
                    .copied()
                    .filter(|s| *s != to)
                    .unwrap_or(0);
                if self.hit[to].is_none() {
                    self.hit[to] = self.hit[self.fail[to]];
                }
                queue.push(to);
            }
        }
    }

    /// Terms found in `text`, deduplicated, in vocabulary order.
    #[must_use]
    pub fn scan(&self, text: &str) -> Vec<String> {
        let mut found = Vec::new();
        let mut node = 0;
        for ch in fold(text).chars() {
            while node != 0 && !self.next[node].contains_key(&ch) {
                node = self.fail[node];
            }
            node = self.next[node].get(&ch).copied().unwrap_or(0);
            if let Some(index) = self.hit[node] {
                let term = self.terms[index].clone();
                if !found.contains(&term) {
                    found.push(term);
                }
            }
        }
        found
    }

    /// Whether this reader would hide the line.
    #[must_use]
    pub fn hides(&self, text: &str) -> bool {
        !self.scan(text).is_empty()
    }
}

/// Lower-case, drop separators, and collapse look-alikes to one form.
///
/// Cyrillic and Latin share a dozen identical-looking letters, and digits
/// stand in for several more. Folding them all the same way — in the
/// vocabulary and in the text alike — means writing around the filter is not a
/// way through it, only a tell.
fn fold(text: &str) -> String {
    text.chars()
        .flat_map(char::to_lowercase)
        .filter_map(|ch| match ch {
            '0' | 'о' => Some('o'),
            '1' | '!' | '|' => Some('i'),
            '3' | 'е' | 'ё' => Some('e'),
            '4' | '@' | 'а' => Some('a'),
            '5' | '$' => Some('s'),
            'р' => Some('p'),
            'с' => Some('c'),
            'х' => Some('x'),
            'у' => Some('y'),
            'к' => Some('k'),
            'м' => Some('m'),
            'т' => Some('t'),
            'в' => Some('b'),
            'н' => Some('h'),
            other if other.is_alphanumeric() => Some(other),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::Screen;

    fn screen() -> Screen {
        Screen::build(&["spam", "дурак"])
    }

    #[test]
    fn a_plain_term_is_found_and_clean_text_is_left_alone() {
        let screen = screen();
        assert_eq!(screen.scan("buy spam now"), vec!["spam".to_owned()]);
        assert!(!screen.hides("a perfectly ordinary sentence"));
        assert!(screen.scan("").is_empty());
    }

    #[test]
    fn writing_around_the_filter_does_not_get_through() {
        let screen = screen();
        assert!(screen.hides("s.p.a.m"), "separators are dropped");
        assert!(screen.hides("SP4M"), "digits fold back to letters");
        assert!(screen.hides("д у р а к"), "spacing is not an escape");
    }

    #[test]
    fn one_pass_finds_several_terms_and_does_not_repeat_them() {
        let screen = screen();
        let found = screen.scan("spam and spam and дурак");
        assert_eq!(found.len(), 2, "{found:?}");
    }

    #[test]
    fn an_empty_vocabulary_hides_nothing() {
        let screen = Screen::build(&[]);
        assert!(!screen.hides("anything at all"));
    }
}
