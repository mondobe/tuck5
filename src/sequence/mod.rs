use std::marker::PhantomData;

use crate::*;
use token::*;

use self::transform::Transform;

#[cfg(test)]
mod tests;

pub trait Sequence<T> {
    fn match_tokens(&self, tokens: &[&Token<T>]) -> Option<usize>;

    fn match_all_tokens(&self, tokens: &[&Token<T>]) -> bool {
        Some(tokens.len()) == self.match_tokens(tokens)
    }

    fn test_and_transform(&self, transform: &impl Transform<T>, tokens: &mut Vec<Token<'_, T>>, start_index: usize) -> Option<usize>
    {
        let test_result = self.match_tokens(&tokens[start_index..].iter().collect::<Vec<_>>());
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

    fn replace_all_matches(&self, transform: impl Transform<T>, tokens: &mut Vec<Token<'_, T>>) -> bool {
        let mut start_index = 0usize;
        let mut changed = false;
        while start_index < tokens.len() {
            let inc = self.test_and_transform(&transform, tokens, start_index);
            if let Some(match_size) = inc {
                start_index += match_size;
                changed = true;
            } else {
                start_index += 1;
            }
        }
        changed
    }
}

pub fn assert_match(seq: impl Sequence<()>, text: &str, should_match: bool) {
    let tox = Token::token_vec_from_str(text, &|_| ());
    assert_eq!(
        seq.match_all_tokens(tox.iter().collect::<Vec<&Token<()>>>().as_slice()),
        should_match
    );
}

pub struct RawSeq<T> {
    pub text: String,
    _t: PhantomData<T>
}

impl <T>RawSeq<T> {
    pub fn new(text: &str) -> RawSeq<T> {
        RawSeq { text: text.to_string(), _t: PhantomData }
    }
}

impl <T>Sequence<T> for RawSeq<T> {
    fn match_tokens(&self, tokens: &[&Token<T>]) -> Option<usize> {
        if tokens.get(0)?.content() == self.text {
            Some(1)
        } else {
            None
        }
    }
}

pub struct FirstTokenSeq<T, F>
where
    F: Fn(&Token<'_, T>) -> bool
{
    pub predicate: F,
    _t: PhantomData<T>
}

impl <T, F>FirstTokenSeq<T, F>
where
    F: Fn(&Token<'_, T>) -> bool
{
    pub fn new(predicate: F) -> FirstTokenSeq<T, F>
    {
        FirstTokenSeq { predicate, _t: PhantomData }
    }
}

impl <T, F>Sequence<T> for FirstTokenSeq<T, F> 
where
    F: Fn(&Token<'_, T>) -> bool
{
    fn match_tokens(&self, tokens: &[&Token<T>]) -> Option<usize>
    {
        if (self.predicate)(tokens.get(0)?) {
            Some(1)
        } else {
            None
        }
    }
}
