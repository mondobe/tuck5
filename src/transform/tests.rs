pub use crate::*;
pub use sequence::*;
use test_case::test_case;
pub use token::*;
pub use transform::*;

#[test_case("abc", "bc"; "removing one letter")]
#[test_case("abca", "bc"; "removing two letters")]
#[test_case("aaa", ""; "removing the same letter repeatedly")]
#[test_case("bc", "bc"; "removing no letters")]
#[test_case("", ""; "no letters")]
pub fn remove_a_test(text: &str, expected: &str) {
    let seq = RawSeq::new("a");
    let tox = &mut Token::token_vec_from_str(text, |_| ());
    let trans = RemoveTransform {};
    replace_all_matches(&seq, &trans, tox);
    assert_eq!(expected, Token::vec_content(tox));
}
