pub use crate::*;
pub use token::*;

use self::transform::Transform;

#[cfg(test)]
pub mod tests;

pub trait Sequence {
    fn match_tokens<T>(&self, tokens: &[&Token<T>]) -> Option<usize>;

    fn match_all_tokens<T>(&self, tokens: &[&Token<T>]) -> bool {
        Some(tokens.len()) == self.match_tokens(tokens)
    }

    fn assert_match(&self, text: &str, should_match: bool) {
        let tox = Token::token_vec_from_str(text, &|_| ());
        assert_eq!(
            self.match_all_tokens(tox.iter().collect::<Vec<&Token<()>>>().as_slice()),
            should_match
        );
    }

    fn test_and_transform<'a, U>(&self, transform: impl Transform<U>, tokens: &mut Vec<Token<'a, U>>, start_index: usize) -> Option<usize>
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
}

pub struct RawSeq {
    pub text: String,
}

impl Sequence for RawSeq {
    fn match_tokens<T>(&self, tokens: &[&Token<T>]) -> Option<usize> {
        if tokens.get(0)?.content() == self.text {
            Some(1)
        } else {
            None
        }
    }
}
