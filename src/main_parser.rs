use chumsky::{prelude::*, text::whitespace};
use crate::types::*;


pub fn parser_of_lextoks<'a>() -> impl Parser<'a,&'a [LexToken], Spanned<SExpr>, extra::Err<chumsky::prelude::Rich<'a,LexToken>>> {
    
    recursive(move |rpars| {
        
        let xany = any::<&'a [LexToken], extra::Err<Rich<LexToken>>>;

        let just = |t:LexTokenType| xany().filter(move |x:&LexToken| x.kind == t);

        let arr = 
            choice((

                just(LexTokenType::OpenBracket)
                .then_ignore(
                    xany().filter(|c:&LexToken|{
                            match c.kind {
                                LexTokenType::Unexpected(_) => true,
                                _ => false
                            }
                        }
                    ).repeated()
                )
                .then(just(LexTokenType::CloseBracket))
                    .map(|(open,close):(LexToken,LexToken)|
                        Spanned(SExpr::List(vec![]),SimpleSpan::new(open.span.start,close.span.end))),
                
                rpars
                    .clone().recover_with(skip_then_retry_until(any().ignored(), end()))
                    .separated_by(
                        just(LexTokenType::Comma).repeated().at_least(1)
                    )
                    .allow_trailing()
                    .collect::<Vec<Spanned<SExpr>>>()
                    .delimited_by(
                        just(LexTokenType::OpenBracket),
                        just(LexTokenType::CloseBracket)
                            .recover_with(skip_then_retry_until(any().ignored(), end()))
                    )
                    .map(|x:Vec<Spanned<SExpr>>|
                        Spanned(SExpr::List(x),SimpleSpan::new(0,0))
                    )
                    .boxed()
            )).labelled("array").as_context();
           
        
        let obj = 
            identifier_with_lextok()
            .then(
                rpars
                    .clone()
                    .repeated()
                    .collect::<Vec<Spanned<SExpr>>>()
            )
            .map(|x|{
                
                let end = match x.1.last() {
                    Some(last_arg) => last_arg.1.end(),
                    None => x.0.1.end,
                };

                Spanned(SExpr::Object(x.0.0, x.1),SimpleSpan::new(x.0.1.start ,end))

            })
            .boxed()            
            .labelled("object").as_context();

        let wrapped_obj = 
            (identifier_with_lextok()
                .then(
                    rpars
                        .repeated()
                        .collect::<Vec<Spanned<SExpr>>>()
                )
            )
            .delimited_by(
                just(LexTokenType::OpenParen),
                just(LexTokenType::CloseParen)
                    .recover_with(skip_then_retry_until(any().ignored(), end()))  
            )
            .map(|(spanned_str,items)|{
                
                let end = match items.last() {
                    Some(last_arg) => last_arg.1.end(),
                    None => spanned_str.1.end,
                };
                Spanned(
                    SExpr::Object(spanned_str.0, items),
                    SimpleSpan::new(spanned_str.1.start, end)
                )

            })
            .boxed().labelled("wrapped object").as_context();
        
        
        let fallback = xany().filter(|x| if let LexTokenType::Unexpected(_) = x.kind {false}else{true}).map(|x| {
            match &x.kind {
                LexTokenType::String(c) => Spanned(SExpr::QuotedString(c.to_string()),x.span),
                LexTokenType::Unexpected(c) => Spanned(SExpr::Unexpected(c.to_string()),x.span),
                _ => Spanned(SExpr::Unexpected(x.to_string()),x.span)
            }
        });
        
        let combo = 
            choice((
                arr.clone().boxed(), 
                obj,
                wrapped_obj,
                
                positive_or_negative_number_lextok().boxed().map(|x:Spanned<f64>|Spanned(SExpr::Number(x.0),x.1)),
                fallback
            ));

            combo
                .clone().validate(|a,b,c|{
                    match &a.0 {
                        SExpr::Unexpected(item) => {
                            c.emit(Rich::custom(a.1,"This is crap"));
                        },
                        _ => {}
                    }
                    a
                })
    })
  
}


fn positive_or_negative_number_lextok<'a>() -> impl Parser<'a,&'a [LexToken], Spanned<f64>, extra::Err<chumsky::prelude::Rich<'a,LexToken>>> {

    let xany = any::<&'a [LexToken], extra::Err<Rich<LexToken>>>;
    let xnum = xany().try_map(|token,_|match token.kind {
        LexTokenType::Number(n) => Ok((n,token.span)),
        _ => Err(Rich::custom(token.span, "expected number"))
    });
    xnum.map(|(a,b):(f64,SimpleSpan)|{
        Spanned(a,b)
    })
}

fn identifier_with_lextok<'a>() -> impl Parser<'a,&'a [LexToken], Spanned<String>, extra::Err<chumsky::prelude::Rich<'a,LexToken>>> {
    
    
    let xany = any::<&'a [LexToken], extra::Err<Rich<LexToken>>>;

    let xident = xany().try_map(|token,_|match token.kind {
        LexTokenType::Identifier(name) => Ok((name,token.span)),
        _ => Err(Rich::custom(token.span, "expected identifier"))
    });

    xident
    .validate(|(identifier_name,span):(String,SimpleSpan),_,emitter|{
        let x = identifier_name.trim();
        if let Some(char_one) = &x.chars().next() {
            let first_char_is_upper_case = char_one.is_ascii_uppercase();
            if !first_char_is_upper_case {
                let tail = x.chars().skip(1).collect::<String>();
                let should_be = format!("{}{}",char_one.to_uppercase(),tail);
                emitter.emit(Rich::custom(span, format!("Did you mean {should_be}?")));
            }
        }
        for (i,c) in x.chars().into_iter().enumerate() {
            if !c.is_alphanumeric() {
                emitter.emit(Rich::custom(SimpleSpan::new(span.start,span.end), format!("Invalid character in identifier: {c}")));                
            }
        }
        Spanned(x.to_string(),SimpleSpan::new(span.start,span.end))
    })
}



