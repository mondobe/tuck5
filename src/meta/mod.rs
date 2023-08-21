use super::*;
use sequence::*;
use transform::*;
use std::{
    collections::HashMap,
    fmt::{Debug, Display, Formatter},
};

#[cfg(test)]
mod tests;

pub fn has_tag<'a>(tag: &'a str) -> impl Sequence<Vec<&str>> {
    FirstTokenSeq::new(move |tok: &Token<'_, Vec<&str>>| tok.data.contains(&tag))
}

pub fn has_tag_owned<'a>(tag: String) -> impl Sequence<Vec<String>> {
    FirstTokenSeq::new(move |tok: &Token<'_, Vec<String>>| tok.data.contains(&tag))
}

pub fn tuck_tokens<'a>(text: &'a str) -> Vec<Token<Vec<&'a str>>> {
    let mut tox = Token::token_vec_from_str(text, |_| vec![]);

    let alphabet = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-";

    replace_all_matches(
        &MultipleSeq::new(vec![
            Box::new(ChooseSeq::from_str(alphabet)),
            Box::new(RepeatedSeq::new(Box::new(ChooseSeq::from_str(alphabet)))),
        ]),
        &ShallowTransform {
            data: vec!["word", "expr"],
        },
        &mut tox,
    );

    replace_all_matches(
        &MultipleSeq::new(vec![
            Box::new(RawSeq::new("'")),
            Box::new(has_tag("word")),
            Box::new(RawSeq::new("'")),
        ]),
        &ShallowTransform {
            data: vec!["seq_ref", "expr"],
        },
        &mut tox,
    );

    let whitespace_seq = FirstTokenSeq::new(|tok: &Token<'_, Vec<&str>>| {
        tok.content()
            .chars()
            .nth(0)
            .is_some_and(|c| c.is_whitespace())
    });

    replace_all_matches(&whitespace_seq, &RemoveTransform {}, &mut tox);

    let func_seq = MultipleSeq::new(vec![
        Box::new(has_tag("word")),
        Box::new(RawSeq::new("(")),
        Box::new(RepeatedSeq::new(Box::new(MultipleSeq::new(vec![
            Box::new(has_tag("expr")),
            Box::new(RawSeq::new(",")),
        ])))),
        Box::new(has_tag("expr")),
        Box::new(RawSeq::new(")")),
    ]);

    repeat_until_no_change(
        &vec![&|c| {
            replace_all_matches(
                &func_seq,
                &DeepTransform {
                    data: vec!["func", "expr"],
                },
                c,
            )
        }],
        &mut tox,
    );

    let def_seq = MultipleSeq::new(vec![
        Box::new(has_tag("seq_ref")),
        Box::new(RawSeq::new("=")),
        Box::new(has_tag("expr")),
    ]);

    replace_all_matches(&def_seq, &DeepTransform { data: vec!["def"] }, &mut tox);

    let rep_deep_seq = MultipleSeq::new(vec![
        Box::new(has_tag("expr")),
        Box::new(RawSeq::new(":")),
        Box::new(MultipleSeq::new(vec![
            Box::new(RepeatedSeq::new(Box::new(MultipleSeq::new(vec![
                Box::new(has_tag("word")),
                Box::new(RawSeq::new(",")),
            ])))),
            Box::new(has_tag("word")),
            Box::new(RawSeq::new(";")),
        ])),
    ]);

    replace_all_matches(
        &rep_deep_seq,
        &DeepTransform {
            data: vec!["rep_leaf", "rep_deep", "rep"],
        },
        &mut tox,
    );

    let rep_remove_seq = MultipleSeq::new(vec![
        Box::new(has_tag("expr")),
        Box::new(RawSeq::new("~")),
    ]);

    replace_all_matches(
        &rep_remove_seq,
        &DeepTransform {
            data: vec!["rep_leaf", "rep_remove", "rep"],
        },
        &mut tox,
    );

    let rep_shallow_seq = MultipleSeq::new(vec![
        Box::new(has_tag("expr")),
        Box::new(RawSeq::new(".")),
        Box::new(MultipleSeq::new(vec![
            Box::new(RepeatedSeq::new(Box::new(MultipleSeq::new(vec![
                Box::new(has_tag("word")),
                Box::new(RawSeq::new(",")),
            ])))),
            Box::new(has_tag("word")),
            Box::new(RawSeq::new(";")),
        ])),
    ]);

    replace_all_matches(
        &rep_shallow_seq,
        &DeepTransform {
            data: vec!["rep_leaf", "rep_shallow", "rep"],
        },
        &mut tox,
    );

    let rep_branch_seq = MultipleSeq::new(vec![
        Box::new(RawSeq::new("{")),
        Box::new(RepeatedSeq::new(
            Box::new(has_tag("rep")),
        )),
        Box::new(RawSeq::new("}")),
    ]);

    replace_all_matches(
        &rep_branch_seq,
        &DeepTransform {
            data: vec!["rep_branch", "rep"],
        },
        &mut tox,
    );

    tox
}

pub fn create_program<'a>(tokens: Vec<Token<'a, Vec<&'a str>>>) -> Option<SeqProg> {
    let mut prog = SeqProg {
        defined_seqs: HashMap::new(),
        reps: vec![],
    };

    for token in tokens {
        if token.data.contains(&"def") {
            if let TokenType::Branch(children) = token.t_type {
                prog.defined_seqs.insert(
                    children[0].content().to_owned(),
                    eval_sequence(children.get(2)?)?,
                );
            }
        } else if token.data.contains(&"rep") {
            prog.reps.push(eval_rep(&token, &prog)?);
        }
    }

    Some(prog)
}

pub fn eval_rep(token: &Token<Vec<&str>>, prog: &SeqProg) -> Option<RepTree> {
    if token.data.contains(&"rep_leaf") {
        if let TokenType::Branch(children) = &token.t_type {
            if token.data.contains(&"rep_remove") {
                if let Some(seq) = eval_sequence(&children[0]) {
                    return Some(RepTree::Leaf(
                        seq.resolve(prog),
                        Box::new(RemoveTransform {})
                    ))
                } else {
                    return None;
                }
            }

            let mut new_tag_tokens = vec![];
            for i in 0usize.. {
                let paren_index = 2 + 2 * i;
                if paren_index >= children.len() {
                    break;
                }
                new_tag_tokens.push(&children[paren_index]);
            }
            if let Some(seq) = eval_sequence(&children[0]) {
                if token.data.contains(&"rep_deep") {
                    Some(RepTree::Leaf(
                        seq.resolve(prog),
                        Box::new(DeepTransform {
                            data: new_tag_tokens.iter().map(|t| 
                                t.content().to_owned()).collect()
                        })
                    ))
                } else if token.data.contains(&"rep_shallow") {
                    Some(RepTree::Leaf(
                        seq.resolve(prog),
                        Box::new(ShallowTransform {
                            data: new_tag_tokens.iter().map(|t| 
                                t.content().to_owned()).collect()
                        })
                    ))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else if token.data.contains(&"rep_branch") {
        if let TokenType::Branch(children) = &token.t_type {
            Some(RepTree::Branch(children[1..children.len() - 1].iter().map(|t| 
                match eval_rep(t, prog) {
                    Some(rt) => Some(rt),
                    None => { return None; }
                }).map(|o| o.unwrap()).collect()))
        } else {
            None
        }
    } else {
        None
    }
}

pub fn eval_sequence(token: &Token<Vec<&str>>) -> Option<DefinedSeq> {
    if token.data.contains(&"seq_ref") {
        Some(DefinedSeq::Ref(token.content().to_owned()))
    } else if token.data.contains(&"func") {
        if let TokenType::Branch(children) = &token.t_type {
            let mut paren_exprs = vec![];
            for i in 0usize.. {
                let paren_index = 2 + 2 * i;
                if paren_index >= children.len() {
                    break;
                }
                paren_exprs.push(&children[paren_index]);
            }
            match children[0].content().to_lowercase().as_str() {
                "raw" => Some(DefinedSeq::Raw(paren_exprs[0].content().to_owned())),
                "choose" => Some(DefinedSeq::Choose(
                    paren_exprs
                        .iter()
                        .map(|e| {
                            Some(match eval_sequence(&e) {
                                Some(ds) => ds,
                                _ => {
                                    return None;
                                }
                            })
                        })
                        .map(|o| o.unwrap())
                        .collect(),
                )),
                "opt" => Some(DefinedSeq::Optional(
                    Box::new(eval_sequence(paren_exprs.get(0)?)?)
                )),
                "repeat" => Some(DefinedSeq::Repeat(
                    Box::new(eval_sequence(paren_exprs.get(0)?)?)
                )),
                "mult" => Some(DefinedSeq::Multiple(
                    paren_exprs
                        .iter()
                        .map(|e| {
                            Some(match eval_sequence(&e) {
                                Some(ds) => ds,
                                _ => {
                                    return None;
                                }
                            })
                        })
                        .map(|o| o.unwrap())
                        .collect(),
                )),
                "has" => Some(DefinedSeq::HasTag(
                    paren_exprs.get(0)?.content().to_owned()
                )),
                _ => None,
            }
        } else {
            None
        }
    } else {
        None
    }
}

pub enum DefinedSeq {
    Ref(String),
    Raw(String),
    Choose(Vec<DefinedSeq>),
    Optional(Box<DefinedSeq>),
    Repeat(Box<DefinedSeq>),
    Multiple(Vec<DefinedSeq>),
    HasTag(String),
}

impl Display for DefinedSeq {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DefinedSeq::Ref(s) => write!(f, "{s}"),
            DefinedSeq::Raw(s) => write!(f, "raw({s})"),
            DefinedSeq::Choose(options) => {
                write!(f, "choose(")?;
                options.iter().for_each(|o| match write!(f, "{o},") {
                    Ok(()) => {},
                    Err(_) => {}
                });
                write!(f, ")")
            },
            DefinedSeq::Optional(d) => write!(f, "opt({})", d.to_string()),
            DefinedSeq::Repeat(d) => write!(f, "repeat({})", d.to_string()),
            DefinedSeq::Multiple(options) => {
                write!(f, "mult(")?;
                options.iter().for_each(|o| match write!(f, "{o},") {
                    Ok(()) => {},
                    Err(_) => {}
                });
                write!(f, ")")
            },
            DefinedSeq::HasTag(s) => write!(f, "has({s})"),
        }
    }
}

impl DefinedSeq {
    pub fn resolve(&self, prog: &SeqProg) -> Box<dyn Sequence<Vec<String>>> {
        match self {
            DefinedSeq::Ref(s) => prog.defined_seqs[s.as_str()].resolve(prog),
            DefinedSeq::Raw(s) => Box::new(RawSeq::new(&s)),
            DefinedSeq::Choose(options) => Box::new(ChooseSeq::new(
                options.iter().map(|d| d.resolve(prog)).collect(),
            )),
            DefinedSeq::Optional(d) => Box::new(OptionalSeq::new(
                d.resolve(prog)
            )),
            DefinedSeq::Repeat(d) => Box::new(RepeatedSeq::new(
                d.resolve(prog)
            )),
            DefinedSeq::Multiple(options) => Box::new(MultipleSeq::new(
                options.iter().map(|d| d.resolve(prog)).collect(),
            )),
            DefinedSeq::HasTag(s) => Box::new(
                has_tag_owned(s.to_owned())
            ),
        }
    }
}

pub struct SeqProg {
    pub defined_seqs: HashMap<String, DefinedSeq>,
    pub reps: Vec<RepTree>,
}

impl SeqProg {
    fn execute(&self, tokens: &mut Vec<Token<Vec<String>>>) {
        for rt in &self.reps {
            rt.execute(self, tokens);
        }
    }
}

impl Display for SeqProg {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        for ds in &self.defined_seqs {
            writeln!(fmt, "{0} = {1}", ds.0, ds.1)?;
        }
        Ok(())
    }
}

impl Debug for SeqProg {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        for ds in &self.defined_seqs {
            writeln!(fmt, "{0} = {1}", ds.0, ds.1)?;
        }
        Ok(())
    }
}

pub enum RepTree {
    Leaf(Box<dyn Sequence<Vec<String>>>, 
        Box<dyn Transform<Vec<String>>>),
    Branch(Vec<RepTree>),
}

impl RepTree {
    pub fn execute(&self, prog: &SeqProg, tokens: &mut Vec<Token<Vec<String>>>) -> bool {
        match self {
            RepTree::Branch(children) => {
                let mut changed = true;
                let mut changed_at_least_once = false;
                while changed {
                    changed = false;
                    if children.iter().any(|f| f.execute(prog, tokens)) {
                        changed = true;
                        changed_at_least_once = true;
                    }
                }
                changed_at_least_once
            },
            RepTree::Leaf(seq, trans) => {
                replace_all_matches(seq.as_ref(), trans.as_ref(), tokens)
            }
        }
    }
}

pub fn char_to_token(c: char) -> Vec<String> {
    let mut to_ret = vec![
        c.to_string(),
        "u".to_owned() + &(c as u32).to_string()
    ];

    if c.is_whitespace() {
        to_ret.push("ws".to_string());
    }

    if c.is_ascii_alphabetic() {
        to_ret.push("alpha".to_string());
    }

    if c.is_ascii_digit() {
        to_ret.push("digit".to_string());
    }

    to_ret
}

pub fn prog_from_str(text: &str) -> Option<SeqProg> {
    create_program(tuck_tokens(text))
}

pub fn eval_prog_from_text<'a>(prog: &str, text: &'a str) -> Vec<Token<'a, Vec<String>>> {
    let sp = prog_from_str(prog).unwrap();
    let mut tox = Token::token_vec_from_str(text, |c| char_to_token(c));
    sp.execute(&mut tox);
    tox
}

pub fn graph_with_tags(tokens: &Vec<Token<Vec<String>>>) {
    for tok in tokens {
        print!("{}", tok.graph());
        print!("Tags: ");
        for tag in &tok.data {
            print!("{} ", tag);
        }
        println!();
    }
}
