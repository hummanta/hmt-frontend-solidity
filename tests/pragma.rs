mod common;

use hmt_frontend_solidity::{ast::*, error::ParseError};

#[test]
fn pragma_bitwise_xor_float() -> Result<(), Vec<ParseError>> {
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
