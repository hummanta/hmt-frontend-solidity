use hmt_frontend_solidity::{ast::*, error::ParseError, parser};

#[test]
fn top_fn_statement_expr() -> Result<(), Vec<ParseError>> {
    let ast = parser::parse("function hello() public { count += 1; }")?;

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
            body: Some(Statement::Block {
                unchecked: false,
                statements: vec![Statement::Expression(Expression::AssignAdd(
                    Box::new(Expression::Variable(Identifier { name: "count".into() })),
                    Box::new(Expression::NumberLiteral("1".into(), None))
                ))]
            })
        })),
    );

    Ok(())
}

#[test]
fn top_fn_statement_return() -> Result<(), Vec<ParseError>> {
    let ast = parser::parse("function hello() public { return count; }")?;

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
            body: Some(Statement::Block {
                unchecked: false,
                statements: vec![Statement::Return(Some(Expression::Variable(Identifier {
                    name: "count".into()
                })))]
            })
        })),
    );

    Ok(())
}
