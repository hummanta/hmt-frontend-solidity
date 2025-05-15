#[macro_export]
macro_rules! parse {
    ($ty:ident, $source:expr) => {{
        use hmt_frontend_solidity::lexer::Lexer;
        paste::expr! { use hmt_frontend_solidity::parser::[<$ty Parser>]; };

        let mut errors = Vec::new();
        let lexer = Lexer::new($source);
        let parser = paste::expr! { [<$ty Parser>]::new() };

        parser.parse(&mut errors, lexer).map_err(|err| {
            errors
                .into_iter()
                .map(ParseError::from)
                .chain(std::iter::once(ParseError::from(err)))
                .collect::<Vec<_>>()
        })
    }};
}
