set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

# -----------------------
# Version (read-only)
# -----------------------

version:
    cargo metadata --format-version 1 \
        | jq -r '.packages[] | select(.name=="confi") | .version'

# -----------------------
# Core workflow
# -----------------------

default:
    @just --list

preflight:
    git diff --quiet || (echo "Working tree not clean" && exit 1)
    git diff --cached --quiet || (echo "Staged changes exist" && exit 1)

    just flake-check
    just check
    just test
    just lint
    just docs
    cargo build --release
    cargo publish --dry-run

test:
   cargo test --no-default-features
   cargo test
   cargo test --all-features

check:
    cargo check --no-default-features
    cargo check
    cargo check --all-features

lint:
    cargo fmt --all -- --check
    cargo clippy --no-default-features --all-targets -- -D warnings
    cargo clippy --all-features --all-targets -- -D warnings

ci:
    just check
    just lint
    just test
    just flake-check

# -----------------------
# Docs
# -----------------------

readme:
    cargo readme > README.md
    git diff --exit-code README.md

docs:
    RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features

# -----------------------
# Coverage (llvm-cov)
# -----------------------

coverage:
    cargo llvm-cov \
        --workspace \
        --all-features \
        --doc \
        --no-report

coverage-ci:
    cargo llvm-cov \
        --workspace \
        --all-features \
        --doc \
        --lcov \
        --output-path lcov.info

# -----------------------
# Dev
# -----------------------

run *args:
    cargo run -- {{args}}

watch:
    cargo watch -x check -x test

bench:
    cargo bench

nix-build:
    nix build

flake-check:
    nix flake check

clean:
    cargo clean

# -----------------------
# Release
# -----------------------

release type:
    just preflight
    just test
    just lint
    just readme

    bash .scripts/release.sh {{type}}
