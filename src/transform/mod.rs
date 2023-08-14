pub use crate::*;
pub use token::*;
pub use sequence::*;

#[cfg(test)]
pub mod tests;

pub trait Transform<T> {
    fn transform<'a>(&self, tokens: Vec<Token<'a, T>>) -> Vec<Token<'a, T>>;
}

pub struct ShallowTransform<T: Clone> {
    data: T
}

impl <T: Clone>Transform<T> for ShallowTransform<T> {
    fn transform<'a>(&self, tokens: Vec<Token<'a, T>>) -> Vec<Token<'a, T>> {
        if let Some(t) = tokens.first() {
            vec![Token {
                root: t.root,
                t_type: TokenType::Leaf(t.content_range()),
                data: self.data.clone()
            }]
        } else {
            vec![]
        }
    }
}

pub fn transform_datas_from_string<T: Clone>(seq: impl Sequence, mut tox: Vec<Token<'_, T>>, start_data: T) -> Vec<T> {
    seq.test_and_transform(ShallowTransform {
        data: start_data
    }, &mut tox, 0);
    tox.iter().map(|t| t.data.clone()).collect()
}