mod common;

use hmt_frontend_solidity::{ast::*, error::ParseError};

#[test]
fn fn_statement_expr() -> Result<(), Vec<ParseError>> {
    let ret = parse!(FunctionDefinition, "function hello() public { count += 1; }")?;

    assert_eq!(
        ret.as_ref(),
        &FunctionDefinition {
            ty: FunctionTy::Function,
            name: Some(Identifier::new("hello")),
            params: vec![],
            attributes: vec![FunctionAttribute::Visibility(Visibility::Public)],
            returns: vec![],
            body: Some(Statement::Block {
                unchecked: false,
                statements: vec![Statement::Expression(Expression::AssignAdd(
                    Box::new(Expression::Variable(Identifier::new("count"))),
                    Box::new(Expression::NumberLiteral("1".into(), None))
                ))]
            })
        },
    );

    Ok(())
}

#[test]
fn fn_statement_return() -> Result<(), Vec<ParseError>> {
    let ret = parse!(FunctionDefinition, "function hello() public { return count; }")?;

    assert_eq!(
        ret.as_ref(),
        &FunctionDefinition {
            ty: FunctionTy::Function,
            name: Some(Identifier::new("hello")),
            params: vec![],
            attributes: vec![FunctionAttribute::Visibility(Visibility::Public)],
            returns: vec![],
            body: Some(Statement::Block {
                unchecked: false,
                statements: vec![Statement::Return(Some(Expression::Variable(Identifier::new(
                    "count"
                ))))]
            })
        },
    );

    Ok(())
}
