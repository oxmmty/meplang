use pest::iterators::Pair;
use pest_derive::Parser;
use crate::parser::error::new_error_from_pair;

#[derive(Parser)]
#[grammar = "meplang.pest"]
pub struct MeplangParser;

pub trait FromPair {
    fn from_pair(expr: Pair<Rule>) -> Result<Self, pest::error::Error<Rule>> where Self: Sized;
}

impl FromPair for bytes::Bytes {
    fn from_pair(hex_litteral: Pair<Rule>) -> Result<Self, pest::error::Error<Rule>> {
        assert!(hex_litteral.as_rule() == Rule::hex_litteral);

        let hex_str = format!(
            "{}{}",
            if hex_litteral.as_str().len() % 2 == 0 {
                ""
            } else {
                "0"
            },
            hex_litteral.as_str().strip_prefix("0x").unwrap(),
        );

        match hex::decode(hex_str) {
            Ok(decoded) => Ok(decoded.into()),
            Err(err) => Err(new_error_from_pair(&hex_litteral, err.to_string())),
        }
    }
}

impl FromPair for String {
    fn from_pair(string_litteral: Pair<Rule>) -> Result<Self, pest::error::Error<Rule>> {
        assert!(string_litteral.as_rule() == Rule::string_litteral);
        
        let mut string_inner = string_litteral.into_inner();
        let res = string_inner.next().unwrap();
        assert!(res.as_rule() == Rule::string_inner);
        assert!(string_inner.next() == None);

        Ok(res.as_str().to_owned())
    }
}
