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

use std::{fs, process};

use anyhow::{bail, Context, Result};
use clap::Parser as _;

use hmt_frontend_solidity::{args::Args, codegen::Codegen, parser};

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

    let ast = match parser::parse(&source) {
        Ok(ast) => ast,
        Err(errors) => {
            let reports = errors
                .iter()
                .map(|err| err.report(&source).context("Failed to generate error report"))
                .collect::<Result<Vec<_>, _>>()?;
            bail!("Parsing failed with {} errors:\n{}", errors.len(), reports.join("\n"));
        }
    };

    if args.print_ast {
        println!("{ast:#?}");
    }

    let mut generator = Codegen::new();
    generator.gen(&ast);
    generator.write(&args.output);

    Ok(())
}
