use hmt_frontend_solidity::{ast::*, error::ParseError, parser};

#[test]
fn simple_empty_contract() -> Result<(), Vec<ParseError>> {
    let ast = parser::parse("contract Counter {}")?;

    assert_eq!(ast.iter().count(), 1);
    let unit = ast.iter().next().unwrap();
    assert_eq!(
        unit,
        &SourceUnit::ContractDefinition(Box::new(ContractDefinition {
            ty: ContractTy::Contract,
            name: Identifier { name: "Counter".to_string() },
            parts: Vec::new()
        }))
    );

    Ok(())
}
