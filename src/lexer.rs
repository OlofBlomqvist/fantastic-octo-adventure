use chumsky::{prelude::*, text::whitespace};

use crate::types::*;


fn positive_or_negative_number<'a>() -> impl Parser<'a,&'a str, f64, extra::Err<chumsky::prelude::Rich<'a,char>>> {
    just('-')
        .or_not()        
        .then(text::int(10))
        .map_slice(|s: &str| s.parse::<f64>().unwrap()).padded()
       
}

fn quoted_string<'a>() -> impl Parser<'a,&'a str, &'a str, extra::Err<chumsky::prelude::Rich<'a,char>>> {
    choice((
        just('"').padded().ignore_then(just('"')).to::<Vec<char>>(vec![]).slice(),
        none_of("\\\"")
            .repeated()
            .delimited_by(
                just('"'), 
                just('"')
            ).slice()
            
    )).padded()
}

fn identifier<'a>() -> impl Parser<'a,&'a str, String, extra::Err<chumsky::prelude::Rich<'a,char>>> {
    
    any().filter(|c:&char| c.is_ascii_alphanumeric() && !c.is_numeric()).then(
        none_of(" ").repeated()
    )
    .slice()
    .padded().map(ToString::to_string)
}


pub fn lexer<'a>() -> impl Parser<'a, &'a str, Vec<LexToken>, extra::Err<Rich<'a, char>>>  {
   
    let identifier = identifier().map(LexTokenType::Identifier);
    let keyword = chumsky::text::unicode::ident().map(|x:&str| LexTokenType::Keyword(x.into()));
    let number = positive_or_negative_number().map(LexTokenType::Number);
    let string = quoted_string().map(|x:&str|LexTokenType::String(x.into()));
    let open_paren = just('(').map(|_| LexTokenType::OpenParen);
    let close_paren = just(')').map(|_| LexTokenType::CloseParen);
    let open_bracket = just('[').map(|_| LexTokenType::OpenBracket);
    let close_bracket = just(']').map(|_| LexTokenType::CloseBracket);
    let comma = just(',').map(|_| LexTokenType::Comma);

    let token = choice((
        identifier,
        keyword,
        number,
        string,
        open_paren,
        close_paren,
        open_bracket,
        close_bracket,
        comma,
        any().map(LexTokenType::Unexpected)
    ))
    .padded()
    .map_with_span(Spanned)
    .map(|x| LexToken { kind:x.0, span:x.1 });

    token.repeated().collect()
 
}



