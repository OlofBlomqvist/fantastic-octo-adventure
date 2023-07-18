use chumsky::{prelude::*, text::whitespace};

use crate::types::*;


use marlowe_lang::types::marlowe::*;

// simplified impl calling the original marlowe_lang ast-node resolver logic
// skipping the walk_all/walk stuff
pub fn haxery<T>(v:AstNode) -> Result<marlowe_lang::parsing::marlowe::ContractParseResult,marlowe_lang::parsing::marlowe::ParseError> {
    match v.try_into() {
        Ok(c) =>  Ok(marlowe_lang::parsing::marlowe::ContractParseResult { 
            uninitialized_time_params: vec![],//v.uninitialized_time_params, 
            uninitialized_const_params: vec![],//v.uninitialized_const_params, 
            contract: c,
            parties: vec![] //v.parties
        }),
        Err(e) => {
            Err(marlowe_lang::parsing::marlowe::ParseError {
                start_line: 0,
                end_line: 0,
                start_col: 0,
                end_col: 0,
                error_message: e,  
            })
        },
    }
}

// This will be outputting ast nodes, and we already have everything we need for converting that.
// Note: we might end up doing some pretty heavy recursion in here, we might want to move to iteration based logic with stack..
pub fn sexpr_into_astnode<'a>() -> impl Parser<'a,&'a [Spanned<SExpr>], Spanned<AstNode>,extra::Err<Rich<'a, Spanned<SExpr>>>> {

    let xany = 
        any::<&'a [Spanned<SExpr>], extra::Err<Rich<Spanned<SExpr>>>>;

    let when_contract = 
        xany().filter(|x|match &x.0 {
            SExpr::Object(name,args) => true,
            _ => false
        })
        .validate(|a,b,c|{
            match &a.0 {
                SExpr::Object(name,args) 
                    if args.len() == 3 
                    && name == "When"
                => Some(a),
                xx => {
                    c.emit(Rich::custom(b,format!("Expected 'When' contract with three arguments, found: {:?}",xx)));
                    None
                }
            }
        })
        .filter(|x|x.is_some()).map(|x|x.unwrap())
        .validate(|a,_,cccc|{
            match a.0 {             
                SExpr::Object(_,args) => {
                    let mut items = args.iter();
                    
                    let _cases = items.next();
                    let _timeout = items.next();
                    let continuation = items.next().unwrap();
                    let contractly = vec![continuation.clone()];
                    
                    let (concrete_continuation,cont_errs) = 
                        sexpr_into_astnode().parse(&contractly).into_output_errors();

                    for x in &cont_errs {
                        cccc.emit(Rich::custom(x.span().clone(), x.to_string()))
                    }

                    let data =  match &concrete_continuation {
                        Some(Spanned(AstNode::MarloweContract(c),_span)) => {
                            let cont = Some(Box::new(c.clone()));
                        
                                    Spanned(
                                        AstNode::MarloweContract(Contract::When { 
                                            when: vec![], 
                                            timeout: None, 
                                            timeout_continuation: cont
                                        }), 
                                        SimpleSpan::new(a.1.start,a.1.end)
                                    )
                        },
                        _ => {
                            return Err(Rich::custom(a.1,"naww.."))
                        }
                    };
                    Ok(data)
                    
                },
                _ => {
                    println!("BOASD");
                    Err(Rich::custom(a.1, "expected close contract. got sad"))
                }
            }
        })
        .try_map(|a,_|{
            a
        })
        .labelled("when_contract").as_context();

    let close_contract = 
        xany().filter(|x|match &x.0 {
            SExpr::Object(name,args) if name =="Close" => true,
            _ => false
        })
        .validate(|a,b,c|{
            match &a.0 {
                SExpr::Object(name,args) => {
                    
                    if name != "Close" {
                        c.emit(Rich::custom(a.1, "Expected Close."));
                    }

                    if args.len() > 0 {
                        c.emit(Rich::custom(a.1, "The 'Close' does not expect any arguments."));
                    }
                },
                _ => {}
            }
            Spanned(AstNode::MarloweContract(Contract::Close),a.1)
            
        })
        
        .labelled("close_contract").as_context();
    
    let contract = 
        choice((
            close_contract,
           when_contract,
        )).labelled("Any Contract").as_context();

    contract
    

}