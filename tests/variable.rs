use hmt_frontend_solidity::{ast::*, error::ParseError, parser};

#[test]
fn def_top_unit_public_var() -> Result<(), Vec<ParseError>> {
    let ast = parser::parse("uint public count;")?;

    assert_eq!(ast.iter().count(), 1);
    let unit = ast.iter().next().unwrap();
    assert_eq!(
        unit,
        &SourceUnit::VariableDefinition(Box::new(VariableDefinition {
            ty: Expression::Variable(Identifier { name: "uint".to_string() }),
            attrs: vec![VariableAttribute::Visibility(Visibility::Public)],
            name: Some(Identifier { name: "count".to_string() }),
            initializer: None
        })),
    );

    Ok(())
}
