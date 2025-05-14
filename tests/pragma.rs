use hmt_frontend_solidity::{ast::*, error::ParseError, parser};

#[test]
fn pragma_bitwise_xor_float() -> Result<(), Vec<ParseError>> {
    let ast = parser::parse("pragma solidity ^0.8;")?;

    assert_eq!(ast.iter().count(), 1);
    let unit = ast.iter().next().unwrap();
    assert_eq!(
        unit,
        &SourceUnit::PragmaDirective(Box::new(PragmaDirective::Version(
            Identifier { name: "solidity".to_string() },
            vec![VersionComparator::Operator { op: VersionOp::Caret, version: "0.8".to_string() }]
        )))
    );

    Ok(())
}
