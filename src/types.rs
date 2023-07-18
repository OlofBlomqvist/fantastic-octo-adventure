use chumsky::span::SimpleSpan;
use nom::lib::std::fmt::Display;

#[derive(Debug, PartialEq,Clone)]
pub struct Spanned<T: std::fmt::Debug>(pub T, pub SimpleSpan<usize>);
impl<T:std::fmt::Debug> Display for Spanned<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let r = format!("{:?}",self.0);
        f.write_str(&r)
    }
}

#[derive(Clone, Debug,PartialEq)]
pub enum LexTokenType {
    Identifier(String),
    Keyword(String),
    Number(f64),
    String(String),
    OpenParen,
    CloseParen,
    OpenBracket,
    CloseBracket,
    Comma,
    Unexpected(char)
}


impl nom::lib::std::fmt::Display for LexToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let r = format!("{:?}",self);
        f.write_str(&r)
    }
}

#[derive(Clone, Debug,PartialEq)]
pub struct LexToken {
    pub kind: LexTokenType,
    pub span: SimpleSpan<usize>,
}


#[derive(Debug, PartialEq,Clone)]
pub enum SExpr {
    QuotedString(String),
    Number(f64),
    List(Vec<Spanned<SExpr>>),
    Object(String,Vec<Spanned<SExpr>>),
    Unexpected(String)
}

