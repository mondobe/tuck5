use std::marker::PhantomData;

use crate::*;
use token::*;

use self::transform::Transform;

#[cfg(test)]
mod tests;

pub trait Sequence<T> {
    fn match_tokens(&self, tokens: &[&Token<T>]) -> Option<usize>;
}

pub fn match_all_tokens<T>(seq: &impl Sequence<T>, tokens: &[&Token<T>]) -> bool {
    Some(tokens.len()) == seq.match_tokens(tokens)
}

pub fn test_and_transform<T>(
    seq: &dyn Sequence<T>,
    transform: &dyn Transform<T>,
    tokens: &mut Vec<Token<'_, T>>,
    start_index: usize,
) -> Option<usize> {
    let test_result = seq.match_tokens(&tokens[start_index..].iter().collect::<Vec<_>>());
    if let Some(len) = test_result {
        let end_index = start_index + len;
        let new_tox = transform.transform(tokens.drain(start_index..end_index).collect());
        let new_len = new_tox.len();
        tokens.splice(start_index..start_index, new_tox);
        Some(new_len)
    } else {
        None
    }
}

pub fn replace_all_matches<T>(
    seq: &dyn Sequence<T>,
    transform: &dyn Transform<T>,
    tokens: &mut Vec<Token<'_, T>>,
) -> bool {
    let mut start_index = 0usize;
    let mut changed = false;
    while start_index < tokens.len() {
        let inc = test_and_transform(seq, transform, tokens, start_index);
        if let Some(match_size) = inc {
            start_index += match_size;
            changed = true;
        } else {
            start_index += 1;
        }
    }
    changed
}

pub fn assert_match(seq: impl Sequence<()>, text: &str, should_match: bool) {
    let tox = Token::token_vec_from_str(text, &|_, _| ());
    assert_eq!(
        match_all_tokens(&seq, tox.iter().collect::<Vec<&Token<()>>>().as_slice()),
        should_match
    );
}

#[derive(Clone)]
pub struct RawSeq<T> {
    pub text: String,
    _t: PhantomData<T>,
}

impl<T> RawSeq<T> {
    pub fn new(text: &str) -> RawSeq<T> {
        RawSeq {
            text: text.to_string(),
            _t: PhantomData,
        }
    }

    pub fn new_from_owned(text: String) -> RawSeq<T> {
        RawSeq {
            text: text.to_string(),
            _t: PhantomData,
        }
    }
}

impl<T> Sequence<T> for RawSeq<T> {
    fn match_tokens(&self, tokens: &[&Token<T>]) -> Option<usize> {
        if tokens.get(0)?.content() == self.text {
            Some(1)
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct FirstTokenSeq<T, F>
where
    F: Fn(&Token<'_, T>) -> bool,
{
    pub predicate: F,
    _t: PhantomData<T>,
}

impl<T, F> FirstTokenSeq<T, F>
where
    F: Fn(&Token<'_, T>) -> bool,
{
    pub fn new(predicate: F) -> FirstTokenSeq<T, F> {
        FirstTokenSeq {
            predicate,
            _t: PhantomData,
        }
    }
}

impl<T, F> Sequence<T> for FirstTokenSeq<T, F>
where
    F: Fn(&Token<'_, T>) -> bool,
{
    fn match_tokens(&self, tokens: &[&Token<T>]) -> Option<usize> {
        if (self.predicate)(tokens.get(0)?) {
            Some(1)
        } else {
            None
        }
    }
}

pub struct ChooseSeq<T> {
    pub options: Vec<Box<dyn Sequence<T>>>,
}

impl<T: 'static> ChooseSeq<T> {
    pub fn new(options: Vec<Box<dyn Sequence<T>>>) -> ChooseSeq<T> {
        ChooseSeq { options }
    }

    pub fn from_str(text: &str) -> ChooseSeq<T> {
        ChooseSeq {
            options: text
                .chars()
                .map(|c| Box::new(RawSeq::new_from_owned(format!("{c}"))) as Box<dyn Sequence<T>>)
                .collect(),
        }
    }
}

impl<T> Sequence<T> for ChooseSeq<T> {
    fn match_tokens(&self, tokens: &[&Token<T>]) -> Option<usize> {
        self.options.iter().find_map(|seq| seq.match_tokens(tokens))
    }
}

pub struct OptionalSeq<T> {
    pub option: Box<dyn Sequence<T>>,
}

impl<T: 'static> OptionalSeq<T> {
    pub fn new(option: Box<dyn Sequence<T>>) -> OptionalSeq<T> {
        OptionalSeq { option }
    }
}

impl<T> Sequence<T> for OptionalSeq<T> {
    fn match_tokens(&self, tokens: &[&Token<T>]) -> Option<usize> {
        match self.option.match_tokens(tokens) {
            None => Some(0),
            Some(len) => Some(len),
        }
    }
}

pub struct RepeatedSeq<T> {
    pub to_repeat: Box<dyn Sequence<T>>,
}

impl<T> RepeatedSeq<T> {
    pub fn new(to_repeat: Box<dyn Sequence<T>>) -> RepeatedSeq<T> {
        RepeatedSeq { to_repeat }
    }
}

impl<T> Sequence<T> for RepeatedSeq<T> {
    fn match_tokens(&self, tokens: &[&Token<T>]) -> Option<usize> {
        let mut index = 0usize;
        loop {
            if index > tokens.len() {
                return Some(tokens.len());
            }

            let res = self.to_repeat.match_tokens(&tokens[index..]);
            if let Some(len) = res {
                index += len;
            } else {
                return Some(index);
            }
        }
    }
}

pub struct MultipleSeq<T> {
    pub seqs: Vec<Box<dyn Sequence<T>>>,
}

impl<T> MultipleSeq<T> {
    pub fn new(seqs: Vec<Box<dyn Sequence<T>>>) -> MultipleSeq<T> {
        MultipleSeq { seqs }
    }
}

impl<T> Sequence<T> for MultipleSeq<T> {
    fn match_tokens(&self, tokens: &[&Token<T>]) -> Option<usize> {
        let mut index = 0usize;
        for seq in &self.seqs {
            if index > tokens.len() {
                return None;
            }

            let res = seq.match_tokens(&tokens[index..]);

            if let Some(len) = res {
                index += len;
            } else {
                return None;
            }
        }
        Some(index)
    }
}
