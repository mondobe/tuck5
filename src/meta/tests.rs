use super::*;
use test_case::test_case;

#[test_case("
'a' = raw(a)
"; "basic raw sequence")]
#[test_case("
'c' = choose(raw(a), raw(b))
"; "basic choose sequence")]
#[test_case("
'c' = choose(choose(raw(a)), raw(b))
"; "basic recursive choose sequence")]
#[test_case("
'c' = opt(raw(a))
"; "basic optional sequence")]
#[test_case("
'c' = repeat(raw(a))
"; "basic repeated sequence")]
#[test_case("
'c' = mult(raw(a), raw(b))
"; "basic multiple sequence")]
pub fn print_simple_prog(prog: &str) {
    let sp = prog_from_str(prog).unwrap();
    dbg!(sp);
}


#[test_case("", "a"; "empty program")]
#[test_case("
raw(a). it_works;
", "a"; "basic raw sequence")]
#[test_case("
'c' = choose(raw(a), raw(b))
'c'. chosen;
", "abac"; "basic choose sequence")]
#[test_case("
'c' = choose(choose(raw(a)), raw(b))
'c'. chosen;
", "abac"; "basic recursive choose sequence")]
#[test_case("
'c' = mult(raw(a), raw(b))
'c'. seq;
", "abac"; "basic multiple sequence")]
#[test_case("
'c' = mult(raw(a), opt(raw(b)))
'c'. seq;
", "abac"; "basic optional sequence")]
#[test_case("
'c' = mult(raw(a), repeat(raw(b)))
'c'. seq;
", "ababbbbac"; "basic repeated sequence")]
#[test_case("
raw(a). foo;
raw(b). bar;
mult(has(foo), has(bar)): baz;
", "baab"; "basic has_tag sequence")]
#[test_case("
has(ws)~;
mult(raw(a), raw(b)). ab;
", "a  
 b"; "basic whitespace removal")]
#[test_case("
raw(a). foo, bar;
", "a"; "more than one tag")]
pub fn eval_simple_prog(prog: &str, text: &str) {
    graph_with_tags(&eval_prog_from_text(prog, text));
}