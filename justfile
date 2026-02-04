# List all available commands
default:
    just --list

# Build the project
build profile="dev" target="":
    cargo build --workspace --all-features --all-targets \
        --profile {{ profile }} {{ if target != "" { "--target " + target } else { "" } }}

# Clean the build artifacts
clean:
    cargo clean --verbose
    rm -rf manifests

# Linting
clippy:
    cargo clippy --workspace --all-features --all-targets -- -D warnings

# Check formatting
check-fmt:
    cargo +nightly fmt --all -- --check

# Fix formatting
fmt:
    cargo +nightly fmt --all

# Test the project
test:
    cargo test --workspace --all-features --all-targets

# Run all the checks
check:
    just check-fmt
    just clippy
    just test

# Install pre-requisites
install:
    just install-hmt-packager
    just install-hmt-manifest

install-hmt-packager:
    cargo install hmt-packager --git https://github.com/hummanta/hummanta --tag v0.11.20

install-hmt-manifest:
    cargo install hmt-manifest --git https://github.com/hummanta/hummanta --tag v0.11.20

# Uninstall pre-requisites
uninstall:
    cargo uninstall hmt-packager
    cargo uninstall hmt-manifest

# Package executables and generate checksums
package profile="dev" target="" version="":
    hmt-packager --profile={{ profile }} --target={{ target }} --version={{ version }}

# Generate the manifests
manifest version="local":
    hmt-manifest  \
      --package hmt-package.toml \
      --artifacts-dir target/artifacts \
      --output-dir manifests \
      --version={{ version }}

# Run all commend in the local environment
all:
    just clean
    just check
    just build dev
    just package dev "" local
    just manifest local

# Bump version in Cargo.toml (interactive)
bump-version:
    #!/usr/bin/env bash
    set -euo pipefail

    # Show current version
    current_version=$(grep -m1 '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
    echo "Current version: $current_version"

    # Prompt for new version
    read -p "New version: " new_version

    # Validate version format
    if ! [[ "$new_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        echo "Error: Version must be in format X.Y.Z (e.g., 0.1.0)"
        exit 1
    fi

    echo ""

    # Update Cargo.toml
    sed -i '' -E \
        "s/^version = \"[0-9]+\.[0-9]+\.[0-9]+\"/version = \"$new_version\"/" \
        Cargo.toml
    echo "Updated Cargo.toml"

    # Run full validation
    echo ""
    echo "Running validation..."
    just all

    echo ""
    echo "Version bump to $new_version completed! Run 'just release' to commit and push."

# Release current version (commit, tag, push)
release:
    #!/usr/bin/env bash
    set -euo pipefail

    # Helper function for confirmation
    confirm() {
        read -p "$1 [y/N] " response
        case "$response" in
            [yY][eE][sS]|[yY]) return 0 ;;
            *) return 1 ;;
        esac
    }

    # Get current version from Cargo.toml
    version=$(grep -m1 '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')

    echo "=== Release v$version ==="
    echo ""

    # Step 1: Git add and commit
    echo "=== [1/3] Git add and commit ==="
    echo "Changes to be committed:"
    git status --short
    echo ""
    if confirm "Run 'git add -A && git commit -m \"chore: bump version to $version\"'?"; then
        git add -A
        git commit -m "chore: bump version to $version"
        echo ""
    else
        echo "Aborted at step 1/3."
        exit 0
    fi

    # Step 2: Git tag
    echo "=== [2/3] Git tag ==="
    if confirm "Run 'git tag -m \"v$version\" v$version'?"; then
        git tag -m "v$version" "v$version"
        echo ""
    else
        echo "Aborted at step 2/3."
        exit 0
    fi

    # Step 3: Push branch and tag
    echo "=== [3/3] Push branch and tag ==="
    if confirm "Run 'git push origin main v$version'?"; then
        git push origin main "v$version"
        echo ""
    else
        echo "Aborted at step 3/3."
        exit 0
    fi

    echo "=== Release v$version completed successfully! ==="
