use std::fmt::Display;

use chumsky::prelude::*;
use ariadne::{Color, Label, Report, ReportKind};
mod types;
mod lexer;
mod main_parser;
mod ast;
use ast::*;
use lexer::*;
use main_parser::*;

fn main() {

    let filename = "example.marlowe";
    let input = std::fs::read_to_string(filename).unwrap();
    
    println!("---- LEXING --------------");
    let (lexer_tokens,lexer_errs) = lexer().parse(&&input).into_output_errors();
   
   if lexer_errs.len() > 0 {
        pretty_print_errors(&input,filename,lexer_errs);
   }

    let lexer_tokens = lexer_tokens.unwrap();
    println!("lex res: {:?}",lexer_tokens);    
    println!("---------------------------------\n");

    println!("---- PARSING LEXED TOKENS IN TO SEXPR TOKENS -----");
    let (parser_outputs,parser_errs)  = 
        parser_of_lextoks().parse(&lexer_tokens).into_output_errors();

    
    if parser_errs.len() > 0 { 
        pretty_print_errors(&input,filename,parser_errs);
        return
     }

    println!("parser output (SEXPR): {:?}",parser_outputs);
    println!("---------------------------------\n");
    
    finalize(&input,filename,parser_outputs);
    
}

fn finalize<'a>(input:&'a str,filename:&'static str,parser_outputs: Option<types::Spanned<types::SExpr>>) {
    if let Some(x) = parser_outputs {
    
        println!("---- Parsing SEXPR in to final AST representation (ASTNODE)");

        let tokens = vec![x];

        let original_parse = sexpr_into_astnode().parse(&tokens);

        let (result,errs) = original_parse.into_output_errors();
        
        pretty_print_errors(input,filename,errs);

        println!("final parser output (AstNode): {:?}",result.clone());
        match result {
            Some(astnode) => {
                let concrete = ast::haxery::<marlowe_lang::types::marlowe::Contract>(astnode.0);
                match concrete {
                    Ok(r) => println!("SUCCESS: {:?}",r.contract),
                    Err(e) => println!("Failed to convert ast node in to contract. {:?}",e),
                }
            },
            None => {
                println!("Could not parse..")
            },
        }
    
        println!("---------------------------------\n");
    }
}





fn pretty_print_errors<'a,E:Display>(input:&'a str,filename:&'static str,errs:Vec<Rich<'a,E>>) {
    
    errs.into_iter()
        .for_each(|e| {
            Report::build(ReportKind::Error, filename.clone(), e.span().start)
                .with_message(e.to_string())
                .with_label(
                    Label::new((filename.clone(), e.span().into_range()))
                        .with_message(e.reason().to_string())
                        .with_color(Color::Red),
                )
                .with_labels(e.contexts().enumerate().map(|(i,(label, span))| {
                    let color = match i {
                        _ => Color::Fixed(255.min(i as u8 + 2))
                    };
                    Label::new((filename.clone(), span.into_range()))
                        .with_message(format!("while parsing this {}", label))
                        .with_color(color)
                }
                ))
                .finish()
                .print(ariadne::sources([(filename.clone(), input.clone())]))
                .unwrap()
        });
}

