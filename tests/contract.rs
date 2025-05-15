mod common;

use hmt_frontend_solidity::{ast::*, error::ParseError};

#[test]
fn simple_empty_contract() -> Result<(), Vec<ParseError>> {
    let ret = parse!(ContractDefinition, "contract Counter {}")?;

    assert_eq!(
        ret.as_ref(),
        &ContractDefinition {
            ty: ContractTy::Contract,
            name: Identifier::new("Counter"),
            parts: Vec::new()
        }
    );

    Ok(())
}
