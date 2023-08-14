use super::*;

pub fn has_tag<'a>(tag: &'a str) -> impl Sequence<Vec<&str>> {
    FirstTokenSeq::new(move |tok: &Token<'_, Vec<&str>>| tok.data.contains(&tag))
}

pub fn calc_tokens<'a>(text: &'a str) -> Vec<Token<Vec<&'a str>>> {
    let mut tox = Token::token_vec_from_str(text, |_| vec![]);

    

    tox
}