# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.11] - 2025-11-05

### Added
- `PhantomCodec` - A codec that encodes/decodes constant values with zero bytes (useful for phantom data)
- `BoxCodec` support in derive macro with `box` keyword
- Phantom codec support in derive macro with `= expr` syntax for constant values
- Improved codec builder syntax - more intuitive pipeline-based approach

### Changed
- **Breaking**: Refactored derive macro syntax for codecs:
  - Arrays: `$arr[item]` → `item $arr`
  - Vectors: `$vec(item)[len]` → `item $vec[len]`
  - Better compose-ability and readability
- Removed `FixedU8SliceCodec` in favor of `BytesCodec` with `PhantomCodec` for fixed lengths
- Updated all examples to use new syntax
- Updated documentation examples across all codec types
- Improved codec table in main library documentation with syntax examples

### Removed
- `FixedU8SliceCodec` (replaced by `BytesCodec(PhantomCodec)`)

## [0.0.10] - 2025-11-04

### Added
- Contributing guidelines (CONTRIBUTING.md)
- Code of Conduct (CODE_OF_CONDUCT.md)
- GitHub Actions CI/CD workflows (Linux-only, stable Rust)
- Issue templates for bugs, features, and questions
- Pull request template
- GitHub Sponsors funding configuration (FUNDING.yml)
- rustfmt.toml for consistent code formatting
- CI badge and community links in README
- Comprehensive Rust documentation (rustdoc) for core types and traits

### Changed
- Formatted codebase with rustfmt
- Enhanced documentation across all modules with detailed examples and usage patterns

## [0.0.9] - 2024-11-03

### Added
- Previous releases before standardized changelog

[Unreleased]: https://github.com/m-ali-akbay/byten/compare/v0.0.11...HEAD
[0.0.11]: https://github.com/m-ali-akbay/byten/compare/v0.0.10...v0.0.11
[0.0.10]: https://github.com/m-ali-akbay/byten/compare/v0.0.9...v0.0.10
[0.0.9]: https://github.com/m-ali-akbay/byten/releases/tag/v0.0.9
