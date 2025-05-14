use hmt_frontend_solidity::{ast::*, error::ParseError, parser};

#[test]
fn def_top_public_function() -> Result<(), Vec<ParseError>> {
    let ast = parser::parse("function hello() public;")?;

    assert_eq!(ast.iter().count(), 1);
    let unit = ast.iter().next().unwrap();
    assert_eq!(
        unit,
        &SourceUnit::FunctionDefinition(Box::new(FunctionDefinition {
            ty: FunctionTy::Function,
            name: Some(Identifier { name: "hello".into() }),
            params: vec![],
            attributes: vec![FunctionAttribute::Visibility(Visibility::Public)],
            returns: vec![],
            body: None
        })),
    );

    Ok(())
}

#[test]
fn top_empty_public_function() -> Result<(), Vec<ParseError>> {
    let ast = parser::parse("function hello() public {}")?;

    assert_eq!(ast.iter().count(), 1);
    let unit = ast.iter().next().unwrap();
    assert_eq!(
        unit,
        &SourceUnit::FunctionDefinition(Box::new(FunctionDefinition {
            ty: FunctionTy::Function,
            name: Some(Identifier { name: "hello".into() }),
            params: vec![],
            attributes: vec![FunctionAttribute::Visibility(Visibility::Public)],
            returns: vec![],
            body: Some(Statement::Block { unchecked: false, statements: vec![] })
        })),
    );

    Ok(())
}

#[test]
fn def_top_public_return_function() -> Result<(), Vec<ParseError>> {
    let ast = parser::parse("function get() public view returns (uint);")?;

    assert_eq!(ast.iter().count(), 1);
    let unit = ast.iter().next().unwrap();
    assert_eq!(
        unit,
        &SourceUnit::FunctionDefinition(Box::new(FunctionDefinition {
            ty: FunctionTy::Function,
            name: Some(Identifier { name: "get".into() }),
            params: vec![],
            attributes: vec![
                FunctionAttribute::Visibility(Visibility::Public),
                FunctionAttribute::Mutability(Mutability::View)
            ],
            returns: vec![Some(Parameter {
                annotation: None,
                ty: Expression::Variable(Identifier { name: "uint".into() }),
                storage: None,
                name: None
            })],
            body: None
        })),
    );

    Ok(())
}
