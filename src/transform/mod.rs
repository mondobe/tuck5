pub use crate::*;
pub use sequence::*;
pub use token::*;

#[cfg(test)]
mod tests;

pub trait Transform<T> {
    fn transform<'a>(&self, tokens: Vec<Token<'a, T>>) -> Vec<Token<'a, T>>;
}

pub struct ShallowTransform<T: Clone> {
    pub data: T,
}

impl<T: Clone> Transform<T> for ShallowTransform<T> {
    fn transform<'a>(&self, tokens: Vec<Token<'a, T>>) -> Vec<Token<'a, T>> {
        if let Some(t) = tokens.first() {
            vec![Token {
                root: t.root,
                t_type: TokenType::Leaf(
                    t.content_range().start..tokens.last().unwrap().content_range().end,
                ),
                data: self.data.clone(),
            }]
        } else {
            vec![]
        }
    }
}

pub struct DeepTransform<T: Clone> {
    pub data: T,
}

impl<T: Clone> Transform<T> for DeepTransform<T> {
    fn transform<'a>(&self, tokens: Vec<Token<'a, T>>) -> Vec<Token<'a, T>> {
        if let Some(t) = tokens.first() {
            vec![Token {
                root: t.root,
                t_type: TokenType::Branch(tokens),
                data: self.data.clone(),
            }]
        } else {
            vec![]
        }
    }
}

pub struct RemoveTransform {}

impl<T> Transform<T> for RemoveTransform {
    fn transform<'a>(&self, _: Vec<Token<'a, T>>) -> Vec<Token<'a, T>> {
        vec![]
    }
}

pub fn repeat_until_no_change(mut func: Vec<Box<dyn FnMut() -> bool>>) -> bool {
    let mut changed = true;
    let mut changed_at_least_once = false;
    while changed {
        changed = false;
        if func.iter_mut().any(|f| f()) {
            changed = true;
            changed_at_least_once = true;
        }
    }
    changed_at_least_once
}
