mod common;

use hmt_frontend_solidity::{ast::*, error::ParseError};

#[test]
fn def_public_var() -> Result<(), Vec<ParseError>> {
    let ret = parse!(VariableDefinition, "uint public count;")?;

    assert_eq!(
        ret.as_ref(),
        &VariableDefinition {
            ty: Expression::Variable(Identifier::new("uint")),
            attrs: vec![VariableAttribute::Visibility(Visibility::Public)],
            name: Some(Identifier::new("count")),
            initializer: None
        },
    );

    Ok(())
}
