// Copyright (c) The Hummanta Authors. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{fs, path::PathBuf, process};

use anyhow::{anyhow, Context, Result};
use ariadne::{Report, Source};
use clap::Parser;

use hmt_frontend_solidity::{codegen::Codegen, diagnostics::ReportToStringExt, parser};

#[derive(Debug, Parser)]
pub struct Args {
    /// Path to the input file
    #[arg(long)]
    pub input: PathBuf,

    /// Path to the output file
    #[arg(long)]
    pub output: PathBuf,

    /// Also print AST to console
    #[arg(long)]
    pub print_ast: bool,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e:?}");
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Args::parse();

    let source = fs::read_to_string(&args.input)
        .context(format!("Failed to read input file: {}", args.input.display()))?;

    // Parse the Solidity source code into an abstract syntax tree (AST).
    // If parsing fails, collect and format all diagnostics into error reports.
    let mut ast = parser::parse(&source, 0).map_err(|diagnostices| {
        let mut reports = Vec::new();
        for diagnostic in diagnostices.iter() {
            let report = Report::from(diagnostic);
            match report.write_to_string(Source::from(&source)) {
                Ok(report_string) => reports.push(report_string),
                Err(e) => return anyhow!("Failed to generate error report: {}", e),
            }
        }
        anyhow!("Parsing failed with {} errors:\n{}", reports.len(), reports.join("\n"))
    })?;

    // Generate the AST representation if requested
    if args.print_ast {
        println!("{ast:#?}");
    }

    // Generate the intermediate representation (IR) from the AST
    // and write it to the output file specified in the arguments
    let mut generator = Codegen::new();
    generator.gen(&mut ast);
    generator.write(&args.output);

    Ok(())
}
