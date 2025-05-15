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
            base: Vec::new(),
            parts: Vec::new()
        }
    );

    Ok(())
}

#[test]
fn simple_base_contract() -> Result<(), Vec<ParseError>> {
    let ret = parse!(ContractDefinition, "contract Counter is A {}")?;

    assert_eq!(
        ret.as_ref(),
        &ContractDefinition {
            ty: ContractTy::Contract,
            name: Identifier::new("Counter"),
            base: vec![Base {
                name: IdentifierPath { identifiers: vec![Identifier::new("A")] },
                args: None
            }],
            parts: Vec::new()
        }
    );

    Ok(())
}

#[test]
fn multi_base_contract() -> Result<(), Vec<ParseError>> {
    let ret = parse!(ContractDefinition, "contract Counter is A, B {}")?;

    assert_eq!(
        ret.as_ref(),
        &ContractDefinition {
            ty: ContractTy::Contract,
            name: Identifier::new("Counter"),
            base: vec![
                Base {
                    name: IdentifierPath { identifiers: vec![Identifier::new("A")] },
                    args: None
                },
                Base {
                    name: IdentifierPath { identifiers: vec![Identifier::new("B")] },
                    args: None
                }
            ],
            parts: Vec::new()
        }
    );

    Ok(())
}

#[test]
fn multi_base_args_contract() -> Result<(), Vec<ParseError>> {
    let ret = parse!(ContractDefinition, r#"contract Counter is A, B, C("x") {}"#)?;

    assert_eq!(
        ret.as_ref(),
        &ContractDefinition {
            ty: ContractTy::Contract,
            name: Identifier::new("Counter"),
            base: vec![
                Base {
                    name: IdentifierPath { identifiers: vec![Identifier::new("A")] },
                    args: None
                },
                Base {
                    name: IdentifierPath { identifiers: vec![Identifier::new("B")] },
                    args: None
                },
                Base {
                    name: IdentifierPath { identifiers: vec![Identifier::new("C")] },
                    args: Some(vec![Expression::StringLiteral(vec![StringLiteral::new(
                        false, "x"
                    )])])
                }
            ],
            parts: Vec::new()
        }
    );

    Ok(())
}

#[test]
fn simple_empty_abstract_contract() -> Result<(), Vec<ParseError>> {
    let ret = parse!(ContractDefinition, "abstract contract Counter {}")?;

    assert_eq!(
        ret.as_ref(),
        &ContractDefinition {
            ty: ContractTy::Abstract,
            name: Identifier::new("Counter"),
            base: Vec::new(),
            parts: Vec::new()
        }
    );

    Ok(())
}

#[test]
fn simple_empty_interface() -> Result<(), Vec<ParseError>> {
    let ret = parse!(ContractDefinition, "interface Counter {}")?;

    assert_eq!(
        ret.as_ref(),
        &ContractDefinition {
            ty: ContractTy::Interface,
            name: Identifier::new("Counter"),
            base: Vec::new(),
            parts: Vec::new()
        }
    );

    Ok(())
}

#[test]
fn simple_empty_library() -> Result<(), Vec<ParseError>> {
    let ret = parse!(ContractDefinition, "library Counter {}")?;

    assert_eq!(
        ret.as_ref(),
        &ContractDefinition {
            ty: ContractTy::Library,
            name: Identifier::new("Counter"),
            base: Vec::new(),
            parts: Vec::new()
        }
    );

    Ok(())
}
