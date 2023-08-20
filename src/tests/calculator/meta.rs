use crate::tests::meta::eval_prog_from_text;

use super::*;
use test_case::test_case;

pub fn calc_tokens<'a>(text: &'a str) -> Vec<Token<'a, Vec<String>>> {
    meta::eval_prog_from_text("
        choose(has(alpha), raw(_)). letter;
        mult(has(letter), repeat(has(letter))). word;
        choose(
            raw(1),
            raw(2),
            raw(3),
            raw(4),
            raw(5),
            raw(6),
            raw(7),
            raw(8),
            raw(9)
        ). nonzero, digit;
        mult(has(nonzero), repeat(has(digit))). int, positive, number, expr;
        raw(0). int, positive, number, expr;
    ", text)
}

pub fn eval(token: &Token<'_, Vec<String>>) -> Option<f64> {
    if !token.data.contains(&"expr".to_string()) {
        None
    } else if token.data.contains(&"parens".to_string()) {
        if let TokenType::Branch(children) = &token.t_type {
            eval(children.get(1)?)
        } else {
            None
        }
    } else if token.data.contains(&"number".to_string()) {
        Some(
            token
                .content()
                .parse()
                .expect("f64 was recognized as correct but didn't parse in Rust"),
        )
    } else if token.data.contains(&"oper".to_string()) {
        if let TokenType::Branch(children) = &token.t_type {
            match children.get(1)?.content() {
                "+" => Some(eval(children.get(0)?)? + eval(children.get(2)?)?),
                "-" => Some(eval(children.get(0)?)? - eval(children.get(2)?)?),
                "*" => Some(eval(children.get(0)?)? * eval(children.get(2)?)?),
                "/" => Some(eval(children.get(0)?)? / eval(children.get(2)?)?),
                _ => None,
            }
        } else {
            None
        }
    } else if token.data.contains(&"call".to_string()) {
        if let TokenType::Branch(children) = &token.t_type {
            match children.first()?.content() {
                "sqrt" => Some(eval(children.get(1)?)?.sqrt()),
                "abs" => Some(eval(children.get(1)?)?.abs()),
                "ln" => Some(eval(children.get(1)?)?.ln()),
                _ => None,
            }
        } else {
            None
        }
    } else {
        None
    }
}

pub fn eval_first(tokens: &Vec<Token<'_, Vec<String>>>) -> Option<f64> {
    if dbg!(tokens).len() < 2 {
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
#[test_case("sqrt(1)", Some(1.0); "basic application")]
#[test_case("sqrt(abs(ln(1)))", Some(0.0); "nested applications")]
#[test_case("sqrt(abs(ln(1)", None; "no trailing end-parens")]
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
