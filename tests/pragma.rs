mod common;

use hmt_frontend_solidity::{ast::*, error::ParseError};

#[test]
fn pragma_identifier() -> Result<(), Vec<ParseError>> {
    let ret = parse!(PragmaDirective, "pragma a b;")?;

    assert_eq!(
        ret,
        PragmaDirective::Identifier(
            Some(Identifier::new("a")), //
            Some(Identifier::new("b"))
        )
    );

    Ok(())
}

#[test]
fn pragma_string_literal() -> Result<(), Vec<ParseError>> {
    let ret = parse!(PragmaDirective, r#"pragma a "b";"#)?;

    assert_eq!(
        ret,
        PragmaDirective::StringLiteral(
            Identifier::new("a"), //
            StringLiteral::new(false, "b")
        )
    );

    Ok(())
}

#[test]
fn pragma_version_plan() -> Result<(), Vec<ParseError>> {
    let ret = parse!(PragmaDirective, "pragma solidity 0.8;")?;

    assert_eq!(
        ret,
        PragmaDirective::Version(
            Identifier::new("solidity"),
            vec![VersionComparator::Plain { version: "0.8".to_string() }]
        )
    );

    Ok(())
}

#[test]
fn pragma_version_operator() -> Result<(), Vec<ParseError>> {
    let ret = parse!(PragmaDirective, "pragma solidity ^0.8;")?;

    assert_eq!(
        ret,
        PragmaDirective::Version(
            Identifier::new("solidity"),
            vec![VersionComparator::Operator { op: VersionOp::Caret, version: "0.8".to_string() }]
        )
    );

    Ok(())
}

#[test]
fn pragma_version_or() -> Result<(), Vec<ParseError>> {
    let ret = parse!(PragmaDirective, "pragma solidity 0.7 || 0.8;")?;

    assert_eq!(
        ret,
        PragmaDirective::Version(
            Identifier::new("solidity"),
            vec![VersionComparator::Or {
                left: Box::new(VersionComparator::Plain { version: "0.7".to_string() }),
                right: Box::new(VersionComparator::Plain { version: "0.8".to_string() })
            }]
        )
    );

    Ok(())
}

#[test]
fn pragma_version_range() -> Result<(), Vec<ParseError>> {
    let ret = parse!(PragmaDirective, "pragma solidity 0.7 - 0.8;")?;

    assert_eq!(
        ret,
        PragmaDirective::Version(
            Identifier::new("solidity"),
            vec![VersionComparator::Range { from: "0.7".to_string(), to: "0.8".to_string() }]
        )
    );

    Ok(())
}
