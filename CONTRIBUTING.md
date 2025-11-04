# Contributing to byten

Thank you for your interest in contributing to byten!

## Code of Conduct

This project follows the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md). Please be respectful and constructive.

## Getting Started

### Development Setup

1. Fork and clone the repository
2. Build: `cargo build`
3. Test: `cargo test`
4. Format: `cargo fmt`
5. Lint: `cargo clippy`

### Project Structure

- `byten/` - Main library crate
- `byten_derive/` - Procedural macro crate

## How to Contribute

### Reporting Issues

Please use GitHub issues to report bugs or suggest features. Include:
- Clear description of the problem or suggestion
- Steps to reproduce (for bugs)
- Rust version (`rustc --version`)

### Pull Requests

Before submitting:
1. Run `cargo fmt` to format your code
2. Run `cargo clippy` and address warnings
3. Run `cargo test` to ensure all tests pass
4. Add tests for new functionality
5. Update documentation as needed

## Style Guidelines

- Follow standard Rust style (enforced by `rustfmt`)
- Write clear commit messages
- Add doc comments for public APIs
- Keep code simple and readable

## Questions?

Open an issue or contact the maintainers at [maakbay@gmail.com](mailto:maakbay@gmail.com).

Thank you for contributing! 🎉
