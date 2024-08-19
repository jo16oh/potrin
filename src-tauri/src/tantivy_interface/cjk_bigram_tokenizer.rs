use cjk::is_cjk_codepoint;
use std::iter::Peekable;
use std::str::CharIndices;
use tantivy::tokenizer::{Token, TokenStream, Tokenizer};

#[derive(Clone, Debug)]
pub struct CJKBigramTokenizer {
    token: Token,
    for_query: bool,
}

impl CJKBigramTokenizer {
    pub fn new() -> CJKBigramTokenizer {
        CJKBigramTokenizer {
            token: Token::default(),
            for_query: false,
        }
    }

    pub fn for_query(mut self) -> Self {
        self.for_query = true;
        self
    }
}

impl Tokenizer for CJKBigramTokenizer {
    type TokenStream<'a> = CJKBigramTokenStream<'a>;
    fn token_stream<'a>(&'a mut self, text: &'a str) -> Self::TokenStream<'a> {
        self.token.reset();

        CJKBigramTokenStream {
            iterator: CJKBigramIterator::new(text, self.for_query),
            text,
            token: &mut self.token,
        }
    }
}

pub struct CJKBigramTokenStream<'a> {
    iterator: CJKBigramIterator<'a>,
    text: &'a str,
    token: &'a mut Token,
}

impl<'a> TokenStream for CJKBigramTokenStream<'a> {
    fn advance(&mut self) -> bool {
        if let Some((offset_from, offset_to)) = self.iterator.next() {
            self.token.position = 0;
            self.token.offset_from = offset_from;
            self.token.offset_to = offset_to;
            self.token.text.clear();
            self.token.text.push_str(&self.text[offset_from..offset_to]);
            true
        } else {
            false
        }
    }

    fn token(&self) -> &Token {
        self.token
    }

    fn token_mut(&mut self) -> &mut Token {
        self.token
    }
}

pub struct CJKBigramIterator<'a> {
    text: &'a str,
    chars: Peekable<CharIndices<'a>>,
    chars_count: usize,
    for_query: bool,
}

impl<'a> CJKBigramIterator<'a> {
    pub fn new(text: &str, for_query: bool) -> CJKBigramIterator {
        CJKBigramIterator {
            text,
            chars: text.char_indices().peekable(),
            chars_count: text.chars().count(),
            for_query,
        }
    }

    fn next_char_endpoint(&mut self) -> Option<usize> {
        let next_char = self.chars.peek();
        match next_char {
            Some((offset, char)) => {
                let len = char.len_utf8();
                Some(offset + len)
            }
            None => None,
        }
    }

    fn get_cjk_bigram(&mut self, offset_from: &usize, char_from: &char) -> Option<(usize, usize)> {
        let next_char = self.chars.peek();
        let offset_to = match next_char {
            Some((offset, char)) => {
                if !char.is_alphanumeric() {
                    Some(*offset)
                } else if is_cjk_codepoint(*char) {
                    self.next_char_endpoint()
                } else {
                    Some(*offset)
                }
            }
            None => None,
        };

        match offset_to {
            Some(offset_to) => Some((*offset_from, offset_to)),
            None => {
                // Return the end character of the CJK sentence,
                // to index it for queries like the following:
                // query: "草"
                // text: "デカすぎで草"
                // When tokenizing queries, we need to omit single end character
                // because Tantivy's query parser combine all tokens as "AND".
                if self.for_query && self.chars_count > 1 {
                    None
                } else {
                    Some((*offset_from, offset_from + char_from.len_utf8()))
                }
            }
        }
    }

    fn get_word(&mut self, offset_from: &usize, char_from: &char) -> Option<(usize, usize)> {
        let next_char = self.chars.peek();
        match next_char {
            Some((offset_to, char_to)) => {
                if char_from.is_alphabetic() && !char_to.is_alphabetic() {
                    Some((*offset_from, *offset_to))
                } else if char_from.is_numeric() && !char_to.is_numeric() {
                    Some((*offset_from, *offset_to))
                } else if is_cjk_codepoint(*char_to) {
                    Some((*offset_from, *offset_to))
                } else {
                    self.chars.next();
                    self.get_word(offset_from, char_from)
                }
            }
            None => Some((*offset_from, self.text.len())),
        }
    }
}

impl<'a> Iterator for CJKBigramIterator<'a> {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<(usize, usize)> {
        let current = self.chars.next();

        match current {
            Some((offset_from, char_from)) => {
                if !char_from.is_alphanumeric() {
                    self.next()
                } else if is_cjk_codepoint(char_from) {
                    self.get_cjk_bigram(&offset_from, &char_from)
                } else {
                    self.get_word(&offset_from, &char_from)
                }
            }

            None => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let text = "私は!「Haneda Airport」に,行きたい。route162 666mafia";
        let mut iterator = CJKBigramIterator::new(text, false);

        let (from, to) = (&mut iterator).next().unwrap();
        assert_eq!("私は", &text[from..to]);
        let (from, to) = (&mut iterator).next().unwrap();
        assert_eq!("は", &text[from..to]);
        let (from, to) = (&mut iterator).next().unwrap();
        assert_eq!("Haneda", &text[from..to]);
        let (from, to) = (&mut iterator).next().unwrap();
        assert_eq!("Airport", &text[from..to]);
        let (from, to) = (&mut iterator).next().unwrap();
        assert_eq!("に", &text[from..to]);
        let (from, to) = (&mut iterator).next().unwrap();
        assert_eq!("行き", &text[from..to]);
        let (from, to) = (&mut iterator).next().unwrap();
        assert_eq!("きた", &text[from..to]);
        let (from, to) = (&mut iterator).next().unwrap();
        assert_eq!("たい", &text[from..to]);
        let (from, to) = (&mut iterator).next().unwrap();
        assert_eq!("い", &text[from..to]);
        let (from, to) = (&mut iterator).next().unwrap();
        assert_eq!("route", &text[from..to]);
        let (from, to) = (&mut iterator).next().unwrap();
        assert_eq!("162", &text[from..to]);
        let (from, to) = (&mut iterator).next().unwrap();
        assert_eq!("666", &text[from..to]);
        let (from, to) = (&mut iterator).next().unwrap();
        assert_eq!("mafia", &text[from..to]);
        assert!((&mut iterator).next().is_none());

        let text = "で草";
        let mut iterator = CJKBigramIterator::new(text, true);
        let (from, to) = (&mut iterator).next().unwrap();
        assert_eq!("で草", &text[from..to]);
        assert!((&mut iterator).next().is_none());

        let text = "で草";
        let mut iterator = CJKBigramIterator::new(text, false);
        let (from, to) = (&mut iterator).next().unwrap();
        assert_eq!("で草", &text[from..to]);
        let (from, to) = (&mut iterator).next().unwrap();
        assert_eq!("草", &text[from..to]);
        assert!((&mut iterator).next().is_none());

        let text = "草";
        let mut iterator = CJKBigramIterator::new(text, true);
        let (from, to) = (&mut iterator).next().unwrap();
        assert_eq!("草", &text[from..to]);
        assert!((&mut iterator).next().is_none());

        let text = "草";
        let mut iterator = CJKBigramIterator::new(text, false);
        let (from, to) = (&mut iterator).next().unwrap();
        assert_eq!("草", &text[from..to]);
        assert!((&mut iterator).next().is_none());
    }
}
