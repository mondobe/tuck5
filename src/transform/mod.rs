pub use crate::*;
pub use token::*;
pub use sequence::*;

#[cfg(test)]
mod tests;

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

pub struct RemoveTransform { }

impl <T>Transform<T> for RemoveTransform {
    fn transform<'a>(&self, _: Vec<Token<'a, T>>) -> Vec<Token<'a, T>> {
        vec![]
    }
}