# sarifup

Command line tool to keep SARIF files updated with historic data.

## Prerequisites

- Rust and Cargo (install via https://rustup.rs). The project uses the 2024 edition; any recent stable Rust toolchain should work.

## Build

To build the release binary:

```bash
cargo build --release
```

The compiled binary will be available at `target/release/sarifup`. 

## Run

The tool reads new SARIF data from stdin and takes the path to an old (historic) SARIF file as a command-line argument.

Examples:

- Using Cargo:
```bash
cat new.sarif | cargo run -- <old.sarif>
```

- Using the compiled binary:
```bash
cat new.sarif | target/release/sarifup <old.sarif>
```

The merged SARIF is printed to stdout.

## Testing

Run unit and integration tests with Cargo:

```bash
cargo test
```
