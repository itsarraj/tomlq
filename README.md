# tomlq

A fast, CLI-based dot-path getter/setter for TOML files. Fills the gap left by `jq` and `yq` in Rust-heavy environments where Cargo configuration files and other TOML files need to be quickly queried or manipulated from shell scripts without manual regex wrangling.

## Status

**built, untested**: the dot-path traversal and modification logic is built on top of `toml_edit`. It has not been heavily tested against complex TOML structures.

## Installation

```sh
cargo install --path .
```

## Usage

```sh
tomlq get package.version Cargo.toml
tomlq set package.version "1.2.3" Cargo.toml
```
