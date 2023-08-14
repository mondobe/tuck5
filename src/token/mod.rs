pub use crate::*;
pub use std::fmt::Write;
pub use std::ops::Range;

#[derive(Debug)]
pub struct Token<'a, T> {
    pub root: &'a str,
    pub t_type: TokenType<'a, T>,
    pub data: T,
}

#[derive(Debug)]
pub enum TokenType<'a, T> {
    Leaf(Range<usize>),
    Branch(Vec<Token<'a, T>>),
}

impl<'a, T> Token<'a, T> {
    pub fn token_vec_from_str(from: &'a str, data: impl Fn(char) -> T) -> Vec<Token<'a, T>> {
        (0..from.len())
            .map(|i| Token {
                t_type: TokenType::Leaf(i..i + 1),
                root: from,
                data: data(from.chars().nth(i).unwrap_or_default()),
            })
            .collect::<Vec<Token<T>>>()
    }

    pub fn content_range(&self) -> Range<usize> {
        match &self.t_type {
            TokenType::Leaf(r) => r.clone(),
            TokenType::Branch(children) => {
                children[0].content_range().start
                    ..children
                        .last()
                        .expect("Branch tokens should have children!")
                        .content_range()
                        .end
            }
        }
    }

    pub fn content(&self) -> &str {
        &self.root[self.content_range()]
    }

    pub fn graph(&self) -> String {
        let mut to_ret = String::new();
        self.graph_depth(0, &mut to_ret)
            .expect("Couldn't graph the token");
        to_ret
    }

    pub fn graph_vec(tox: &Vec<Token<T>>) -> String {
        tox.iter()
            .map(|i| i.graph())
            .fold(String::new(), |l, r| format!("{l}\n{r}"))
    }

    pub fn vec_content(tox: &Vec<Token<T>>) -> String {
        tox.iter()
            .map(|i| i.content())
            .fold(String::new(), |l, r| format!("{l}{r}"))
    }

    pub fn graph_depth(&self, depth: usize, buf: &mut String) -> Result<(), std::fmt::Error> {
        for _ in 0..depth {
            write!(buf, "\t")?;
        }
        match &self.t_type {
            TokenType::Branch(children) => {
                writeln!(buf, "{{")?;
                for child in children {
                    child.graph_depth(depth + 1, buf)?;
                }
                for _ in 0..depth {
                    write!(buf, "\t")?;
                }
                writeln!(buf, "}}")?;
            }
            TokenType::Leaf(_) => {
                writeln!(buf, "{}", self.content())?;
            }
        }

        Ok(())
    }
}
