use chumsky::prelude::*;

enum Expr {
    Let{
        name:&str
    },
}

fn parser<'src>() -> impl Parser<'src, &'src str, ()> {
    let ident = text::ascii::ident().padded();
    let decl = recursive(|decl| {
        let r#let = text::ascii::keyword("let")
            .ignore_then(ident)
            .then_ignore(just('='))
            .then(expr.clone())
            .then_ignore(just(';'))
            .then(decl)
            .map(|((name, rhs), then)| Expr::Let {
                name,
                rhs: Box::new(rhs),
                then: Box::new(then),
            });

        r#let
            // Must be later in the chain than `r#let` to avoid ambiguity
            .or(expr)
            .padded()
    });

    decl
}

#[test]
fn test_parser() {
    // Our parser expects empty strings, so this should parse successfully
    assert_eq!(parser().parse("").into_result(), Ok(()));

    // Anything other than an empty string should produce an error
    assert!(parser().parse("123").has_errors());
}
