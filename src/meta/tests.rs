use super::*;
use test_case::test_case;

#[test_case("", "a"; "empty program")]
#[test_case("
raw(a). it_works;
", "a"; "basic raw sequence")]
#[test_case("
choose(raw(a), raw(b)). chosen;
", "abac"; "basic choose sequence")]
#[test_case("
choose(choose(raw(a)), raw(b)). chosen;
", "abac"; "basic recursive choose sequence")]
#[test_case("
mult(raw(a), raw(b)). seq;
", "abac"; "basic multiple sequence")]
#[test_case("
mult(raw(a), opt(raw(b))). seq;
", "abac"; "basic optional sequence")]
#[test_case("
mult(raw(a), repeat(raw(b))). seq;
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
#[test_case("
raw(a). foo;
raw(b). bar;
foo & bar: baz;
", "baab"; "sugared has_tag sequence")]
#[test_case("
'a'. it_works;
", "a"; "sugared raw sequence")]
#[test_case("
a..c. it_works;
", "abcd"; "sugared range sequence")]
#[test_case("
a+. it_works;
", "a"; "sugared one or more sequence")]
#[test_case("
# pre-comment
raw(a) # inline comment
. it_works;
", "a"; "comments")]
#[test_case("
## pre-comment with misleading code
'a'. b;
##
raw(a) ## inline comment ##
. it_works;
", "a"; "long comments")]
#[test_case("
{.quote.}. quote;
", "quotea"; "quotes")]
pub fn eval_simple_prog(prog: &str, text: &str) {
    graph_with_tags(&eval_prog_from_text(prog, text));
}
