use crate::meta::eval_prog_from_text;

use super::*;
use test_case::test_case;

pub fn calc_tokens<'a>(text: &str) -> Vec<Token<Vec<String>>> {
    meta::eval_prog_from_text(
        "
         ## recognize words
            (e.g. sqrt, abs)
         ##
        a..z | A..Z | '_'. letter;
        letter+. word;

         ## recognize digits and positive integers
            (integers cannot have leading zeroes - maybe change this? It's
            mainly to show off that it's possible...)
         ##
        1..9. nonzero, digit;
        0. digit;
        nonzero & digit*. int, positive, number, expr;
        '0'. int, positive, number, expr;

          # recognize decimals and negative numbers
        int & '.' & int+. decimal, positive, number, expr;
        '-' & positive: negative, number, expr;

          # remove whitespace
        ws~;

         ## The members of \"PEMDAS\" you know and love, along with function
            calls.
         ##
        {
            '(' & expr & ')': parens, expr;
            word & parens: call, expr;
            expr & '*' | '/' & expr: oper, expr;
            expr & '+' | '-' & expr: oper, expr;
        }
    ",
        text,
    )
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
#[test_case("((((50)))) * 1 + 1", Some(51.0); "order of operations with nested parens, backwards")]
#[test_case("sqrt(1)", Some(1.0); "basic application")]
#[test_case("sqrt(abs(ln(1)))", Some(0.0); "nested applications")]
#[test_case("sqrt(abs(ln(1)", None; "no trailing end-parens")]
pub fn eval_test(text: &str, expected: Option<f64>) {
    assert_eq!(eval_text(text), expected)
}
