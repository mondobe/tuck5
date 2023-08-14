pub use crate::*;
pub use sequence::*;
use test_case::test_case;
pub use token::*;

#[test_case("a", true; "basic passing case")]
#[test_case("b", false; "basic failing case")]
#[test_case("", false; "matching empty string")]
#[test_case("aa", false; "multiple of correct token")]
pub fn raw_seq_test(text: &str, should_match: bool) {
    let seq = RawSeq::new("a");
    assert_match(seq, text, should_match);
}

#[test_case("a", true; "first option")]
#[test_case("b", true; "second option")]
#[test_case("", false; "matching empty string")]
#[test_case("aa", false; "multiple of correct token")]
#[test_case("ab", false; "both correct tokens")]
#[test_case("c", false; "single incorrect token")]
pub fn choose_seq_test(text: &str, should_match: bool) {
    let seq = ChooseSeq::new(vec![Box::new(RawSeq::new("a")), Box::new(RawSeq::new("b"))]);
    assert_match(seq, text, should_match);
}

#[test_case("a", true; "basic passing case")]
#[test_case("", true; "empty passing case")]
#[test_case("b", false; "incorrect string")]
#[test_case("aa", false; "multiple of incorrect token")]
pub fn opt_seq_test(text: &str, should_match: bool) {
    let seq = OptionalSeq::new(Box::new(RawSeq::new("a")));
    assert_match(seq, text, should_match);
}

#[test_case("a", true; "basic passing case")]
#[test_case("", true; "empty passing case")]
#[test_case("aa", true; "multiple of incorrect token")]
#[test_case("aab", false; "multiple of correct token followed by one incorrect")]
#[test_case("b", false; "incorrect string")]
pub fn rep_seq_test(text: &str, should_match: bool) {
    let seq = RepeatedSeq::new(Box::new(RawSeq::new("a")));
    assert_match(seq, text, should_match);
}

#[test_case("ab", true; "basic passing case")]
#[test_case("", false; "empty failing case")]
#[test_case("aa", false; "multiple of incorrect token")]
#[test_case("aab", false; "multiple of correct token followed by one incorrect")]
#[test_case("b", false; "incorrect string")]
pub fn mult_seq_test(text: &str, should_match: bool) {
    let seq = MultipleSeq::new(vec![Box::new(RawSeq::new("a")), Box::new(RawSeq::new("b"))]);
    assert_match(seq, text, should_match);
}
