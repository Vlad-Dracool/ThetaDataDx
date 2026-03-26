# Contributing to thetadatadx

Thank you for your interest in contributing to thetadatadx. This guide covers everything
you need to get started.

## Prerequisites

- **Rust stable** (see `rust-toolchain.toml` — includes rustfmt and clippy)
- **protoc** (Protocol Buffers compiler) — only needed if modifying `.proto` files
- **Python 3.9+** — for the Python SDK
- **maturin** — for building the PyO3 Python bindings (`pip install maturin`)
- **Go 1.21+** — for the Go SDK
- **cargo-deny** — for dependency auditing (`cargo install cargo-deny`)

## Development Setup

```bash
git clone https://github.com/userFRM/ThetaDataDx.git
cd thetadatadx

# Run the full workspace test suite
cargo test --workspace

# For integration tests against ThetaData servers, create creds.txt:
# Line 1: email
# Line 2: password
```

## Code Style

All Rust code must pass formatting and linting checks:

```bash
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
```

## Pre-commit Checks

Run the full CI-equivalent check locally before pushing:

```bash
# Core workspace
cargo fmt --all -- --check && cargo test --workspace && cargo clippy --workspace -- -D warnings

# FFI crate
cargo build --release -p thetadatadx-ffi

# Python SDK (if modified)
cd sdks/python && maturin develop && python -m pytest
```

Do not push code that fails any of these checks. CI will reject it.

## How to Add a New Endpoint

1. **Update the proto definition** (if the endpoint uses a new message type)
   - Edit the relevant `.proto` file under `crates/thetadatadx/proto/`
   - Regenerate Rust types with `cargo build` (prost build script handles codegen)

2. **Add the method to DirectClient**
   - Add the request/response method in `crates/thetadatadx/src/mdds/`
   - Follow the pattern of existing endpoints (request builder, response parser)
   - Add unit tests

3. **Expose in the Python SDK**
   - Add the PyO3 binding in `sdks/python/src/`
   - Add a Python test in `sdks/python/tests/`

4. **Expose in the FFI layer**
   - Add the C ABI function in `ffi/src/`
   - Update the C header if needed
   - Update Go and C++ SDK wrappers accordingly

5. **Update CHANGELOG.md** under `[Unreleased]`

## How to Update After a ThetaData Terminal Update

When ThetaData releases a new terminal version that changes the wire protocol:

1. Refer to `docs/reverse-engineering.md` for methodology
2. Capture new traffic and compare against existing FIT/FIE codec expectations
3. Update frame parsers, tick types, or message definitions as needed
4. Run the full test suite to verify backwards compatibility

## Running Against Dev Servers

Set environment variables to point at a non-production terminal:

```bash
export THETADX_HOST="127.0.0.1"
export THETADX_PORT="11000"
cargo test --workspace
```

## Pull Request Process

1. **Branch** — create a feature branch from `main` (`feat/description` or `fix/description`)
2. **Test** — run the full pre-commit check suite (see above)
3. **PR** — open a pull request against `main`
4. **Review** — address reviewer feedback; all CI checks must pass
5. **Merge** — squash-merge preferred for clean history

Every PR must include:
- Passing CI (fmt, clippy, test, deny)
- Updated `CHANGELOG.md` if user-facing
- Updated documentation if any public API changed

## Community

Join the ThetaData Discord for questions and discussion: **[discord.thetadata.us](https://discord.thetadata.us/)**

## Code of Conduct

This project follows the [Contributor Covenant v2.1](CODE_OF_CONDUCT.md).
Be respectful, constructive, and professional in all interactions.
