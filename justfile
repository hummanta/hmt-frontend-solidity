# List all available commands
default:
    just --list

# Build the project
build profile="dev" target="":
    RUST_BACKTRACE=1 cargo build --workspace --all-features --tests --bins --benches \
        --profile {{profile}} {{ if target != "" { "--target " + target } else { "" } }}

# Clean the build artifacts
clean:
    cargo clean --verbose
    rm -rf manifests

# Linting
clippy:
   cargo clippy --workspace --all-features --tests --bins --benches -- -D warnings

# Check formatting
fmt:
    cargo +nightly fmt --all -- --check

# Test the project
test:
    RUST_BACKTRACE=1 cargo test --workspace --all-features --verbose

# Run all the checks
check:
    just fmt
    just clippy
    just test

# Install pre-requisites
install:
    just install-hmt-packager
    just install-hmt-manifest

install-hmt-packager:
    cargo install hmt-packager --git https://github.com/hummanta/hummanta --tag v0.11.4

install-hmt-manifest:
    cargo install hmt-manifest --git https://github.com/hummanta/hummanta --tag v0.11.4

# Uninstall pre-requisites
uninstall:
    cargo uninstall hmt-packager
    cargo uninstall hmt-manifest

# Package executables and generate checksums
package profile="dev" target="" version="":
    hmt-packager --profile={{profile}} --target={{target}} --version={{version}}

# Generate the manifests
manifest version="local":
    hmt-manifest  \
      --package hmt-package.toml \
      --artifacts-dir target/artifacts \
      --output-dir manifests \
      --version={{version}}

# Run all commend in the local environment
all:
    just clean
    just check
    just build dev
    just package dev "" local
    just manifest local
