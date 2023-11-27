use super::*;
use sequence::*;
use std::fmt::{Debug, Display, Formatter};
use transform::*;

#[cfg(test)]
mod tests;

pub fn has_tag<'a>(tag: &'a str) -> impl Sequence<Vec<&str>> {
    FirstTokenSeq::new(move |tok: &Token<'_, Vec<&str>>| tok.data.contains(&tag))
}

pub fn has_tag_owned<'a>(tag: String) -> impl Sequence<Vec<String>> {
    FirstTokenSeq::new(move |tok: &Token<'_, Vec<String>>| tok.data.contains(&tag))
}

pub fn raw_range<'a>(s: u32, e: u32) -> impl Sequence<Vec<String>> {
    FirstTokenSeq::new(move |tok: &Token<'_, Vec<String>>| {
        tok.content()
            .chars()
            .next()
            .is_some_and(|c| (s..=e).contains(&(c as u32)))
            && tok.content().len() == 1
    })
}

pub fn tuck_tokens<'a>(text: &'a str) -> Vec<Token<Vec<&'a str>>> {
    let mut tox = Token::token_vec_from_str(text, |_, _| vec![]);

    replace_all_matches_once(
        &MultipleSeq::new(vec![
            Box::new(RawSeq::new("'")),
            Box::new(RepeatedSeq::new(Box::new(FirstTokenSeq::new(|t| {
                t.content() != "'"
            })))),
            Box::new(RawSeq::new("'")),
        ]),
        &ShallowTransform {
            data: vec!["raw", "expr"],
        },
        &mut tox,
    );

    replace_all_matches_once(
        &MultipleSeq::new(vec![
            Box::new(RawSeq::new("#")),
            Box::new(RawSeq::new("#")),
            Box::new(RepeatedSeq::new(Box::new(FirstTokenSeq::new(|t| {
                t.content() != "#"
            })))),
            Box::new(RawSeq::new("#")),
            Box::new(RawSeq::new("#")),
        ]),
        &RemoveTransform {},
        &mut tox,
    );

    replace_all_matches_once(
        &MultipleSeq::new(vec![
            Box::new(RawSeq::new("#")),
            Box::new(RepeatedSeq::new(Box::new(FirstTokenSeq::new(|t| {
                t.content() != "\n"
            })))),
            Box::new(RawSeq::new("\n")),
        ]),
        &RemoveTransform {},
        &mut tox,
    );

    replace_all_matches_once(
        &MultipleSeq::new(vec![
            Box::new(RawSeq::new("{")),
            Box::new(RawSeq::new(".")),
            Box::new(RepeatedSeq::new(Box::new(FirstTokenSeq::new(|t| {
                t.content() != "."
            })))),
            Box::new(RawSeq::new(".")),
            Box::new(RawSeq::new("}")),
        ]),
        &ShallowTransform {
            data: vec!["quote", "expr"],
        },
        &mut tox,
    );

    replace_all_matches_once(
        &MultipleSeq::new(vec![
            Box::new(FirstTokenSeq::new(|t| t.content() != ".")),
            Box::new(RawSeq::new(".")),
            Box::new(RawSeq::new(".")),
            Box::new(FirstTokenSeq::new(|t| t.content() != ".")),
        ]),
        &ShallowTransform {
            data: vec!["range", "expr"],
        },
        &mut tox,
    );

    let alphabet = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-";

    replace_all_matches_once(
        &MultipleSeq::new(vec![
            Box::new(ChooseSeq::from_str(alphabet)),
            Box::new(RepeatedSeq::new(Box::new(ChooseSeq::from_str(alphabet)))),
        ]),
        &ShallowTransform {
            data: vec!["word", "expr"],
        },
        &mut tox,
    );

    let whitespace_seq = FirstTokenSeq::new(|tok: &Token<'_, Vec<&str>>| {
        tok.content()
            .chars()
            .nth(0)
            .is_some_and(|c| c.is_whitespace())
    });

    replace_all_matches_once(&whitespace_seq, &RemoveTransform {}, &mut tox);

    let opt_seq: MultipleSeq<Vec<&str>> =
        MultipleSeq::new(vec![Box::new(has_tag("expr")), Box::new(RawSeq::new("?"))]);

    let repeat_seq = MultipleSeq::new(vec![Box::new(has_tag("expr")), Box::new(RawSeq::new("*"))]);

    let one_or_more_seq =
        MultipleSeq::new(vec![Box::new(has_tag("expr")), Box::new(RawSeq::new("+"))]);

    let mult_seq = MultipleSeq::new(vec![
        Box::new(has_tag("expr")),
        Box::new(RawSeq::new("&")),
        Box::new(RepeatedSeq::new(Box::new(MultipleSeq::new(vec![
            Box::new(has_tag("expr")),
            Box::new(RawSeq::new("&")),
        ])))),
        Box::new(has_tag("expr")),
    ]);

    let choose_seq = MultipleSeq::new(vec![
        Box::new(has_tag("expr")),
        Box::new(RawSeq::new("|")),
        Box::new(RepeatedSeq::new(Box::new(MultipleSeq::new(vec![
            Box::new(has_tag("expr")),
            Box::new(RawSeq::new("|")),
        ])))),
        Box::new(has_tag("expr")),
    ]);

    let paren_seq = MultipleSeq::new(vec![
        Box::new(RawSeq::new("(")),
        Box::new(has_tag("expr")),
        Box::new(RawSeq::new(")")),
    ]);

    repeat_until_no_change(
        &vec![
            &|c| {
                replace_first_match(
                    &paren_seq,
                    &DeepTransform {
                        data: vec!["parens", "expr"],
                    },
                    c,
                )
            },
            &|c| {
                replace_first_match(
                    &opt_seq,
                    &DeepTransform {
                        data: vec!["opt", "expr"],
                    },
                    c,
                )
            },
            &|c| {
                replace_first_match(
                    &repeat_seq,
                    &DeepTransform {
                        data: vec!["repeat", "expr"],
                    },
                    c,
                )
            },
            &|c| {
                replace_first_match(
                    &one_or_more_seq,
                    &DeepTransform {
                        data: vec!["one_or_more", "expr"],
                    },
                    c,
                )
            },
            &|c| {
                replace_first_match(
                    &choose_seq,
                    &DeepTransform {
                        data: vec!["choose", "expr"],
                    },
                    c,
                )
            },
            &|c| {
                replace_first_match(
                    &mult_seq,
                    &DeepTransform {
                        data: vec!["mult", "expr"],
                    },
                    c,
                )
            },
        ],
        &mut tox,
    );

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

    let rep_remove_seq = MultipleSeq::new(vec![
        Box::new(has_tag("expr")),
        Box::new(RawSeq::new("~")),
        Box::new(RawSeq::new(";")),
    ]);

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

    let rep_once_seq = MultipleSeq::new(vec![Box::new(RawSeq::new("%")), Box::new(has_tag("rep"))]);

    let rep_branch_seq = MultipleSeq::new(vec![
        Box::new(RawSeq::new("{")),
        Box::new(RepeatedSeq::new(Box::new(has_tag("no_once")))),
        Box::new(RawSeq::new("}")),
    ]);

    repeat_until_no_change(
        &vec![
            &|c| {
                replace_all_matches_once(
                    &rep_deep_seq,
                    &DeepTransform {
                        data: vec!["rep_leaf", "rep_deep", "rep", "no_once"],
                    },
                    c,
                )
            },
            &|c| {
                replace_all_matches_once(
                    &rep_shallow_seq,
                    &DeepTransform {
                        data: vec!["rep_leaf", "rep_shallow", "rep", "no_once"],
                    },
                    c,
                )
            },
            &|c| {
                replace_all_matches_once(
                    &rep_remove_seq,
                    &DeepTransform {
                        data: vec!["rep_leaf", "rep_remove", "rep", "no_once"],
                    },
                    c,
                )
            },
            &|c| {
                replace_all_matches_once(
                    &rep_once_seq,
                    &DeepTransform {
                        data: vec!["once", "rep"],
                    },
                    c,
                )
            },
            &|c| {
                replace_all_matches_once(
                    &rep_branch_seq,
                    &DeepTransform {
                        data: vec!["rep_branch", "rep"],
                    },
                    c,
                )
            },
        ],
        &mut tox,
    );

    tox
}

pub fn create_program<'a>(tokens: Vec<Token<'a, Vec<&'a str>>>) -> Option<SeqProg> {
    let mut prog = SeqProg { reps: vec![] };

    for token in tokens {
        if token.data.contains(&"rep") {
            prog.reps.push(eval_rep(&token, &prog)?);
        }
    }

    Some(prog)
}

pub fn eval_rep(token: &Token<Vec<&str>>, prog: &SeqProg) -> Option<RepTree> {
    if token.data.contains(&"once") {
        Some(RepTree::Once(Box::new(eval_rep(
            token.nth_child(1)?,
            prog,
        )?)))
    } else if token.data.contains(&"rep_leaf") {
        if let TokenType::Branch(children) = &token.t_type {
            if token.data.contains(&"rep_remove") {
                if let Some(seq) = eval_sequence(&children[0]) {
                    return Some(RepTree::Leaf(seq, Box::new(RemoveTransform {})));
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
                        seq,
                        Box::new(DeepTransform {
                            data: new_tag_tokens
                                .iter()
                                .map(|t| t.content().to_owned())
                                .collect(),
                        }),
                    ))
                } else if token.data.contains(&"rep_shallow") {
                    Some(RepTree::Leaf(
                        seq,
                        Box::new(ShallowTransform {
                            data: new_tag_tokens
                                .iter()
                                .map(|t| t.content().to_owned())
                                .collect(),
                        }),
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
            Some(RepTree::Branch(
                children[1..children.len() - 1]
                    .iter()
                    .map(|t| match eval_rep(t, prog) {
                        Some(rt) => Some(rt),
                        None => {
                            return None;
                        }
                    })
                    .map(|o| o.unwrap())
                    .collect(),
            ))
        } else {
            None
        }
    } else {
        None
    }
}

pub fn eval_sequence(token: &Token<Vec<&str>>) -> Option<Box<dyn Sequence<Vec<String>>>> {
    if token.data.contains(&"mult") {
        if let TokenType::Branch(children) = &token.t_type {
            let mut paren_exprs = vec![];
            for i in 0usize.. {
                let paren_index = 2 * i;
                if paren_index >= children.len() {
                    break;
                }
                paren_exprs.push(&children[paren_index]);
            }
            Some(Box::new(MultipleSeq::new(
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
            )))
        } else {
            None
        }
    } else if token.data.contains(&"choose") {
        if let TokenType::Branch(children) = &token.t_type {
            let mut paren_exprs = vec![];
            for i in 0usize.. {
                let paren_index = 2 * i;
                if paren_index >= children.len() {
                    break;
                }
                paren_exprs.push(&children[paren_index]);
            }
            Some(Box::new(ChooseSeq::new(
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
            )))
        } else {
            None
        }
    } else if token.data.contains(&"parens") {
        if let TokenType::Branch(children) = &token.t_type {
            eval_sequence(children.get(1)?)
        } else {
            None
        }
    } else if token.data.contains(&"opt") {
        if let TokenType::Branch(children) = &token.t_type {
            Some(Box::new(OptionalSeq::new(eval_sequence(children.get(0)?)?)))
        } else {
            None
        }
    } else if token.data.contains(&"repeat") {
        if let TokenType::Branch(children) = &token.t_type {
            Some(Box::new(RepeatedSeq::new(eval_sequence(children.get(0)?)?)))
        } else {
            None
        }
    } else if token.data.contains(&"one_or_more") {
        if let TokenType::Branch(children) = &token.t_type {
            Some(Box::new(MultipleSeq::new(vec![
                eval_sequence(children.get(0)?)?,
                Box::new(RepeatedSeq::new(eval_sequence(children.get(0)?)?)),
            ])))
        } else {
            None
        }
    } else if token.data.contains(&"raw") {
        Some(Box::new(RawSeq::new(
            &token.content()[1..token.content().len() - 1],
        )))
    } else if token.data.contains(&"quote") {
        Some(Box::new(MultipleSeq::new(
            token.content()[2..token.content().len() - 2]
                .chars()
                .map(|c| Box::new(RawSeq::new(&c.to_string())) as Box<dyn Sequence<Vec<String>>>)
                .collect(),
        )))
    } else if token.data.contains(&"range") {
        Some(Box::new(raw_range(
            token.content().chars().nth(0)? as u32,
            token.content().chars().nth(3)? as u32,
        )))
    } else if token.data.contains(&"word") {
        Some(Box::new(has_tag_owned(token.content().to_string())))
    } else {
        None
    }
}

pub enum DefinedSeq {
    Raw(String),
    Range(u32, u32),
    Choose(Vec<DefinedSeq>),
    Optional(Box<DefinedSeq>),
    Repeat(Box<DefinedSeq>),
    Multiple(Vec<DefinedSeq>),
    HasTag(String),
}

impl Display for DefinedSeq {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DefinedSeq::Raw(s) => write!(f, "raw({s})"),
            DefinedSeq::Range(s, e) => write!(f, "{s}..{e}"),
            DefinedSeq::Choose(options) => {
                write!(f, "choose(")?;
                options.iter().for_each(|o| match write!(f, "{o},") {
                    Ok(()) => {}
                    Err(_) => {}
                });
                write!(f, ")")
            }
            DefinedSeq::Optional(d) => write!(f, "opt({})", d.to_string()),
            DefinedSeq::Repeat(d) => write!(f, "repeat({})", d.to_string()),
            DefinedSeq::Multiple(options) => {
                write!(f, "mult(")?;
                options.iter().for_each(|o| match write!(f, "{o},") {
                    Ok(()) => {}
                    Err(_) => {}
                });
                write!(f, ")")
            }
            DefinedSeq::HasTag(s) => write!(f, "has({s})"),
        }
    }
}

impl DefinedSeq {
    pub fn resolve(&self, prog: &SeqProg) -> Box<dyn Sequence<Vec<String>>> {
        match self {
            DefinedSeq::Raw(s) => Box::new(RawSeq::new(&s)),
            DefinedSeq::Range(s, e) => Box::new(raw_range(*s, *e)),
            DefinedSeq::Choose(options) => Box::new(ChooseSeq::new(
                options.iter().map(|d| d.resolve(prog)).collect(),
            )),
            DefinedSeq::Optional(d) => Box::new(OptionalSeq::new(d.resolve(prog))),
            DefinedSeq::Repeat(d) => Box::new(RepeatedSeq::new(d.resolve(prog))),
            DefinedSeq::Multiple(options) => Box::new(MultipleSeq::new(
                options.iter().map(|d| d.resolve(prog)).collect(),
            )),
            DefinedSeq::HasTag(s) => Box::new(has_tag_owned(s.to_owned())),
        }
    }
}

pub struct SeqProg {
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
        write!(
            fmt,
            "{}",
            self.reps
                .iter()
                .map(|rt| format!("{rt}"))
                .reduce(|acc, i| acc + &i)
                .unwrap_or(String::new())
        )
    }
}

impl Debug for SeqProg {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(fmt, "{self}")
    }
}

pub enum RepTree {
    Leaf(
        Box<dyn Sequence<Vec<String>>>,
        Box<dyn Transform<Vec<String>>>,
    ),
    Branch(Vec<RepTree>),
    Once(Box<RepTree>),
}

impl RepTree {
    pub fn execute(&self, prog: &SeqProg, tokens: &mut Vec<Token<Vec<String>>>) -> bool {
        match self {
            RepTree::Branch(children) => {
                let mut changed_at_least_once = false;
                let mut i = 0;
                'outer: while i < children.len() {
                    let changed = children[i].execute(prog, tokens);
                    if changed {
                        i = 0;
                        changed_at_least_once = true;
                        continue 'outer;
                    }
                    i += 1;
                }
                changed_at_least_once
            }
            RepTree::Leaf(seq, trans) => replace_first_match(seq.as_ref(), trans.as_ref(), tokens),
            RepTree::Once(rep) => execute_once(rep, prog, tokens),
        }
    }
}

fn execute_once(rep: &RepTree, prog: &SeqProg, tokens: &mut Vec<Token<Vec<String>>>) -> bool {
    let rep = rep;
    match rep {
        RepTree::Leaf(seq, trans) => replace_all_matches_once(seq.as_ref(), trans.as_ref(), tokens),
        RepTree::Branch(children) => {
            for t in children {
                execute_once(t, prog, tokens);
            }
            true
        }
        RepTree::Once(rep) => rep.execute(prog, tokens),
    }
}

impl Display for RepTree {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self {
            RepTree::Branch(children) => write!(
                fmt,
                "{}",
                children
                    .iter()
                    .map(|rt| format!("{rt}"))
                    .reduce(|acc, i| acc + &i)
                    .unwrap_or(String::new())
            ),
            RepTree::Leaf(_, _) => write!(fmt, "sequence"),
            RepTree::Once(r) => write!(fmt, "%{r}"),
        }
    }
}

impl Debug for RepTree {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(fmt, "{self}")
    }
}

pub fn char_to_token(c: char) -> Vec<String> {
    let mut to_ret = vec![c.to_string(), "u".to_owned() + &(c as u32).to_string()];

    if c.is_whitespace() || c.is_control() {
        to_ret.push("ws".to_string());
    }

    to_ret
}

pub fn prog_from_str(text: &str) -> Option<SeqProg> {
    create_program(tuck_tokens(text))
}

pub fn eval_prog_from_text<'a>(prog: &str, text: &'a str) -> Vec<Token<'a, Vec<String>>> {
    let sp = prog_from_str(prog).unwrap();
    let mut tox = Token::token_vec_from_str(text, |r, i| char_to_token(r.chars().nth(i).unwrap()));
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

pub fn graph_with_tags_b(tokens: &Vec<Token<Vec<&str>>>) {
    for tok in tokens {
        print!("{}", tok.graph());
        print!("Tags: ");
        for tag in &tok.data {
            print!("{} ", tag);
        }
        println!();
    }
}
