use super::*;
use test_case::test_case;

pub fn has_tag<'a>(tag: &'a str) -> impl Sequence<Vec<&str>> {
    FirstTokenSeq::new(move |tok: &Token<'_, Vec<&str>>| tok.data.contains(&tag))
}

pub fn calc_tokens<'a>(text: &'a str) -> Vec<Token<Vec<&'a str>>> {
    let mut tox = Token::token_vec_from_str(text, |_| vec![]);

    replace_all_matches(
        &int_seq(),
        ShallowTransform {
            data: vec!["int", "positive", "number", "expr"],
        },
        &mut tox,
    );

    let decimal_seq = MultipleSeq::new(vec![
        Box::new(has_tag("int")),
        Box::new(RawSeq::new(".")),
        Box::new(ChooseSeq::from_str("0123456789")),
        Box::new(RepeatedSeq::new(Box::new(ChooseSeq::from_str(
            "0123456789",
        )))),
    ]);

    replace_all_matches(
        &decimal_seq,
        ShallowTransform {
            data: vec!["decimal", "positive", "number", "expr"],
        },
        &mut tox,
    );

    let negative_seq = MultipleSeq::new(vec![
        Box::new(RawSeq::new("-")),
        Box::new(has_tag("positive")),
    ]);

    replace_all_matches(
        &negative_seq,
        DeepTransform {
            data: vec!["negative", "number", "expr"],
        },
        &mut tox,
    );

    let whitespace_seq = FirstTokenSeq::new(|tok: &Token<'_, Vec<&str>>| {
        tok.content()
            .chars()
            .nth(0)
            .is_some_and(|c| c.is_whitespace())
    });

    let paren_seq = MultipleSeq::new(vec![
        Box::new(RawSeq::new("(")),
        Box::new(has_tag("expr")),
        Box::new(RawSeq::new(")")),
    ]);

    let md_seq = MultipleSeq::new(vec![
        Box::new(has_tag("expr")),
        Box::new(ChooseSeq::from_str("*/")),
        Box::new(has_tag("expr")),
    ]);

    let as_seq = MultipleSeq::new(vec![
        Box::new(has_tag("expr")),
        Box::new(ChooseSeq::from_str("+-")),
        Box::new(has_tag("expr")),
    ]);

    replace_all_matches(&whitespace_seq, RemoveTransform {}, &mut tox);

    repeat_until_no_change(&[
        &|c| replace_all_matches(&paren_seq, 
            DeepTransform { data: vec!["parens", "expr"] }, 
            c),
        &|c| replace_all_matches(&md_seq, 
            DeepTransform { data: vec!["oper", "expr"] }, 
            c),
        &|c| replace_all_matches(&as_seq, 
            DeepTransform { data: vec!["oper", "expr"] }, 
            c)
    ], &mut tox);

    tox
}

pub fn eval(token: &Token<'_, Vec<&str>>) -> Option<f64> {
    if !token.data.contains(&"expr") {
        return None;
    }

    if token.data.contains(&"parens") {
        if let TokenType::Branch(children) = &token.t_type {
            return eval(children.get(1)?);
        }
    }

    if token.data.contains(&"number") {
        return Some(
            token
                .content()
                .parse()
                .expect("f64 was recognized as correct but didn't parse in Rust"),
        );
    }

    if token.data.contains(&"oper") {
        if let TokenType::Branch(children) = &token.t_type {
            return match children.get(1)?.content() {
                "+" => Some(eval(children.get(0)?)? + eval(children.get(2)?)?),
                "-" => Some(eval(children.get(0)?)? - eval(children.get(2)?)?),
                "*" => Some(eval(children.get(0)?)? * eval(children.get(2)?)?),
                "/" => Some(eval(children.get(0)?)? / eval(children.get(2)?)?),
                _ => None
            };
        } else {
            return None;
        }
    }

    None
}

pub fn eval_first(tokens: &Vec<Token<'_, Vec<&str>>>) -> Option<f64> {
    if tokens.len() < 2 {
        eval(tokens.first()?)
    } else {
        None
    }
}

pub fn eval_text(text: &str) -> Option<f64> {
    eval_first(&calc_tokens(text))
}

#[test_case("1", Some(1.0); "one-digit number")]
#[test_case("a", None; "basic letter fail")]
#[test_case("", None; "no text")]
#[test_case(" 101", Some(101.0); "leading space")]
#[test_case("0101", None; "leading zero")]
#[test_case("123", Some(123.0); "multiple-digit number")]
#[test_case("123.0", Some(123.0); "multiple-digit number with decimal")]
#[test_case("0", Some(0.0); "zero")]
#[test_case("-1", Some(-1.0); "negative integer")]
#[test_case("-123.0", Some(-123.0); "negative decimal")]
#[test_case("1 + 1", Some(2.0); "basic integer addition")]
#[test_case("1 - 1", Some(0.0); "basic integer subtracting")]
#[test_case("2 * 3", Some(6.0); "basic integer multiplication")]
#[test_case("6 / 3", Some(2.0); "basic integer division")]
#[test_case("1 + 2 * 3 + 1", Some(8.0); "order of operations")]
#[test_case("(1)", Some(1.0); "basic parenthetical")]
#[test_case("1 + 1 * ((((50))))", Some(51.0); "order of operations with nested parens")]
pub fn eval_test(text: &str, expected: Option<f64>) {
    assert_eq!(eval_text(text), expected)
}

pub fn int_seq() -> impl Sequence<Vec<&'static str>> {
    let nonzero_digit = ChooseSeq::from_str("123456789");
    let digit = ChooseSeq::from_str("0123456789");
    ChooseSeq::new(vec![
        Box::new(MultipleSeq::new(vec![
            Box::new(nonzero_digit),
            Box::new(RepeatedSeq::new(Box::new(digit))),
        ])),
        Box::new(RawSeq::new("0")),
    ])
}

#[test_case("1", true; "one-digit int")]
#[test_case("1234567890", true; "all digits")]
#[test_case("0", true; "special case")]
#[test_case("01", false; "leading zero")]
pub fn int_test(text: &str, should_match: bool) {
    let tox = Token::token_vec_from_str(text, &|_| vec![]);
    assert_eq!(
        match_all_tokens(
            int_seq(),
            tox.iter().collect::<Vec<&Token<Vec<&str>>>>().as_slice()
        ),
        should_match
    );
}
