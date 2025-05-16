# hmt-frontend-solidity

Parsing the Solidity source code and generating the Cranelift IR,
This parser is compatible with Ethereum Solidity
[v0.8.22](https://docs.soliditylang.org/en/v0.8.22/grammar.html).

## Installation

Prebuilt binaries Windows, Linux and macOS can be downloaded from the
[Github release page](https://github.com/hummanta/hmt-frontend-solidity/releases/latest).
If there is no distro package available in your preferred manager,
you need [Rust and cargo](https://www.rust-lang.org/tools/install) to build it.

### Install from source:

1. Clone the repository with `git clone
   https://github.com/hummanta/hmt-frontend-solidity.git`
2. From the `hmt-frontend-solidity` directory, run `cargo build --release` to
   build the application in release mode.
3. After a successful compilation, launch the executable with:
   `target/release/hmt-frontend-solidity`.

### Install with cargo

To get the latest bug fixes and features, you can install the development
version from git. However, this is not fully tested. That means you're probably
going to have more bugs despite having the latest bug fixes.

```
cargo install --git https://github.com/hummanta/hmt-frontend-solidity
```

This will download the source from the main branch, build and install it in
Cargo's global binary directory (`~/.cargo/bin/` by default).

## Usage

```text
Usage: hmt-frontend-solidity [OPTIONS] --input <INPUT> --output <OUTPUT>

Options:
      --input <INPUT>    Path to the input file
      --output <OUTPUT>  Path to the output file
      --print-ast        Also print AST to console
  -h, --help             Print help
```

Example:

```bash
hmt-frontend-solidity --input solidity/counter/counter.sol --ouput counter.clif
```

## Development

To build this project, you will need to install the following pre-requisites:
[Git](https://git-scm.com/downloads),
[Rust](https://www.rust-lang.org/tools/install) and
[Just](https://github.com/casey/just).

After cloning the repository, you can simply run `just` in the package directory
to list all available commands. For your first local build, please run `just
install` command to install the dependencies for this project.

## Contributing

If anything feels off, or if you feel that some functionality is missing, please
check out the [contributing page](CONTRIBUTING.md). There you will find
instructions for sharing your feedback, building the project locally, and
submitting pull requests to the project.

## License

Copyright (c) The Hummanta Authors. All rights reserved.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

The parser in this project is heavily based on or copied from
[solang-parser](https://github.com/hyperledger/solang).
This project also follows the Apache 2.0 license for collaboration.
