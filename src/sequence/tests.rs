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
