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

    replace_all_matches(&whitespace_seq, RemoveTransform {}, &mut tox);

    tox
}

pub fn eval(token: &Token<'_, Vec<&str>>) -> Option<f64> {
    if !token.data.contains(&"expr") {
        return None;
    }

    if token.data.contains(&"number") {
        return Some(
            token
                .content()
                .parse()
                .expect("f64 was recognized as correct but didn't parse in Rust"),
        );
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

#[test_case("1", Some(1.0); "one-digit number")]
#[test_case("123", Some(123.0); "multiple-digit number")]
#[test_case("123.0", Some(123.0); "multiple-digit number with decimal")]
#[test_case("0", Some(0.0); "zero")]
#[test_case("-1", Some(-1.0); "negative integer")]
#[test_case("-123.0", Some(-123.0); "negative decimal")]
pub fn eval_test(text: &str, expected: Option<f64>) {
    assert_eq!(eval_first(&calc_tokens(text)), expected)
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
