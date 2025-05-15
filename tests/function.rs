mod common;

use hmt_frontend_solidity::{ast::*, error::ParseError};

#[test]
fn def_public_function() -> Result<(), Vec<ParseError>> {
    let ret = parse!(FunctionDefinition, "function hello() public;")?;

    assert_eq!(
        ret.as_ref(),
        &FunctionDefinition {
            ty: FunctionTy::Function,
            name: Some(Identifier::new("hello")),
            params: vec![],
            attributes: vec![FunctionAttribute::Visibility(Visibility::Public)],
            returns: vec![],
            body: None
        }
    );

    Ok(())
}

#[test]
fn empty_public_function() -> Result<(), Vec<ParseError>> {
    let ret = parse!(FunctionDefinition, "function hello() public {}")?;

    assert_eq!(
        ret.as_ref(),
        &FunctionDefinition {
            ty: FunctionTy::Function,
            name: Some(Identifier::new("hello")),
            params: vec![],
            attributes: vec![FunctionAttribute::Visibility(Visibility::Public)],
            returns: vec![],
            body: Some(Statement::Block { unchecked: false, statements: vec![] })
        }
    );

    Ok(())
}

#[test]
fn def_public_return_function() -> Result<(), Vec<ParseError>> {
    let ret = parse!(FunctionDefinition, "function get() public view returns (uint);")?;

    assert_eq!(
        ret.as_ref(),
        &FunctionDefinition {
            ty: FunctionTy::Function,
            name: Some(Identifier::new("get")),
            params: vec![],
            attributes: vec![
                FunctionAttribute::Visibility(Visibility::Public),
                FunctionAttribute::Mutability(Mutability::View)
            ],
            returns: vec![Some(Parameter {
                annotation: None,
                ty: Expression::Variable(Identifier::new("uint")),
                storage: None,
                name: None
            })],
            body: None
        }
    );

    Ok(())
}
