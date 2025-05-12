// Copyright (c) The Hummanta Authors. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::iter::once;

use lalrpop_util::lalrpop_mod;

use crate::{ast::Program, error::ParseError, lexer::Lexer};

lalrpop_mod!(grammar);

/// Parses source into Program or returns syntax errors
pub fn parse(source: &str) -> Result<Program, Vec<ParseError>> {
    let lexer = Lexer::new(source);
    let parser = grammar::ProgramParser::new();
    let mut errors = Vec::new(); // Collected during parse

    parser.parse(&mut errors, lexer).map_err(|err| {
        errors
            .into_iter() // Take errors
            .map(Into::into) // Convert type
            .chain(once(err.into())) // Add final error
            .collect()
    })
}

#[cfg(test)]
mod test {
    use lalrpop_util::{ErrorRecovery, ParseError as LalrpopParseError};

    use crate::{
        ast::*,
        error::{LexicalError, ParseError},
        parser,
        token::Token,
    };

    #[test]
    fn test_parse_var() -> Result<(), Vec<ParseError>> {
        let ast = parser::parse("var x = 42;")?;

        assert_eq!(ast.0.len(), 1);

        let statement = ast.0.first().unwrap();
        assert_eq!(
            statement,
            &Statement::Variable(VarStatement {
                name: String::from("x"),
                value: Box::new(Expression::Integer(42))
            })
        );

        Ok(())
    }

    #[test]
    fn test_error_recovery() {
        let expected: ErrorRecovery<usize, Token, LexicalError> = ErrorRecovery {
            error: LalrpopParseError::UnrecognizedToken {
                token: (6, Token::Error, 7),
                expected: vec!["\"=\"".to_string()],
            },
            dropped_tokens: vec![],
        };

        if let Err(errors) = parser::parse("var x != 42;") {
            assert_eq!(errors, vec![expected.into()])
        }
    }
}
