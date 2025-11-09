# byten

[![Crates.io](https://img.shields.io/crates/v/byten.svg)](https://crates.io/crates/byten)
[![Documentation](https://docs.rs/byten/badge.svg)](https://docs.rs/byten)
[![License](https://img.shields.io/crates/l/byten.svg)](https://github.com/m-ali-akbay/byten#license)

A binary codec library for efficient encoding and decoding of Rust data structures.

> ⚠️ **Early Development**: This library is in active development and the API may change.

## Features

- 🚀 **Derive macros** for automatic codec implementation (`Encode`, `Decode`, `Measure`)
- 🔢 **Primitive types** with custom byte ordering (BE/LE) and variable-length encoding
- 📦 **Collections** support: `Vec`, arrays, slices with configurable length prefixes
- 🔤 **String handling**: UTF-8 strings, C strings (`CStr`), and byte slices
- 💾 **Zero-copy decoding** with borrowed data (`'encoded` lifetime)
- 🎯 **Type-safe** enums with discriminant encoding
- 🔧 **Flexible** attribute-based customization with inline codec syntax
- 🎨 **Nested structures** including boxed types for recursive data

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
byten = "0.0"
```

## Basic Usage

```rust
use byten::{DefaultCodec, DecodeOwned, Encode, Measure, EncodeToVec as _};

#[derive(Debug, DefaultCodec, Encode, Measure, DecodeOwned, PartialEq)]
struct Person {
    #[byten($be)]
    id: u32,
    age: u8,
    #[byten($bytes[u8] $utf8 $own)]
    name: String,
}

fn main() {
    let person = Person {
        id: 12345,
        age: 30,
        name: "Alice".to_string(),
    };

    // Encode to Vec
    let encoded = person.encode_to_vec().unwrap();
    
    // Decode from slice
    let mut offset = 0;
    let decoded = Person::decode(&encoded, &mut offset).unwrap();
    assert_eq!(person, decoded);
}
```

## Advanced Examples

### Zero-Copy Borrowed Data

```rust
use byten::{DefaultCodec, Decode, Encode, Measure, EncodeToVec as _};
use std::ffi::CStr;

#[derive(Debug, DefaultCodec, Encode, Decode, Measure)]
pub struct Person<'encoded> {
    pub first_name: &'encoded CStr,
    pub last_name: &'encoded CStr,
    
    #[byten($bytes[u16 $be] $utf8)]
    pub address: &'encoded str,
    
    #[byten($bytes[u32 $uvarbe])]
    pub avatar_image: &'encoded [u8],
    
    #[byten(.. $utf8)]
    pub extra_data: &'encoded str,
}
```

### Enums with Discriminants

```rust
use byten::{DefaultCodec, DecodeOwned, Encode, Measure};

#[derive(Debug, DefaultCodec, DecodeOwned, Encode, Measure, PartialEq)]
#[repr(u16)]
#[byten($le)]
enum Color {
    Red = 1,
    Green = 2,
    Blue = 3,
    Grayscale(#[byten($be)] u16) = 4,
    RGBa {
        red: u8,
        green: u8,
        blue: u8,
        #[byten($be)]
        alpha: u16,
    } = 5,
    Gradient(Box<Color>, Box<Color>) = 6,
}
```

### Recursive Structures

```rust
use byten::{DefaultCodec, DecodeOwned, Encode, Measure};
use std::ffi::CString;

#[derive(Debug, DefaultCodec, Encode, Measure, DecodeOwned)]
pub struct Directory {
    #[byten(CStr $own)]
    pub name: CString,
    #[byten(Entry box for[u16 $be])]
    pub entries: Vec<Box<Entry>>,
}

#[derive(Debug, DefaultCodec, Encode, Measure, DecodeOwned)]
pub struct File {
    #[byten(CStr $own)]
    pub name: CString,
    #[byten($bytes[u16 $be] $own)]
    pub content: Vec<u8>,
    #[byten(u32 $be ?)]
    pub assigned_application_id: Option<u32>,
}

#[derive(Debug, DefaultCodec, Encode, Measure, DecodeOwned)]
#[repr(u8)]
pub enum Entry {
    File(File) = 1,
    Directory(Directory) = 2,
}
```

### Inline Codec Syntax

```rust
use byten::{byten, Decoder, EncoderToVec as _};

fn main() {
    // Define codec inline without derive macros
    let codec = byten!( $bytes[u32 $be] $utf8 );

    let original_str = "Hello, Byten!";
    let encoded = codec.encode_to_vec(original_str).unwrap();
    let decoded_str = codec.decode(&encoded, &mut 0).unwrap();
}
```

## Attribute Syntax

The `#[byten(...)]` attribute supports a flexible syntax for customizing encoding:

- **Endianness**: `$be` (big-endian), `$le` (little-endian)
- **Variable-length**: `$uvarbe` (variable-length unsigned, big-endian)
- **Collections**: `T for[Length]`, `T []`
- **Bytes**: `$bytes[Length]` for raw byte slices
- **Strings**: `$utf8` for UTF-8 strings, `CStr` for C strings
- **Ownership**: `$own` to decode into owned data (e.g., `String`, `Vec`)
- **Optional**: `?` for `Option<T>` types with presence byte
- **Boxing**: `box` for `Box<T>` types
- **Phantom**: `= expr` for constant values with zero bytes
- **Remaining**: `..` to consume rest of input
- **Tuples**: `(a, b, ...N)` as built-in codecs for N sized tuples
- **Custom**: `{ expr }` for custom codec expressions

## Features Flags

- `std` (default): Enable standard library support. Disable for `no_std` environments with `default-features = false`
- `alloc` (default via `std`): Enable types that require allocation (`Vec`, `Box`, `String`). Can be used in `no_std` with an allocator.
- `anyhow` (default): Integration with the `anyhow` error handling crate (requires `std`)
- `derive` (default): Enable derive macros for self-coded traits. Works in all modes (core-only, alloc, std).

### Using in `no_std` Environments

#### Without an allocator (core only)

For embedded systems without an allocator, use only core types:

```toml
[dependencies]
byten = { version = "0.0", default-features = false }
```

This provides support for:
- Primitive types (`u8`, `i8`, `bool`, etc.)
- Arrays and slices
- Borrowed data (`&str`, `&[u8]`, `&CStr`)
- Endian conversion
- Fixed-size types

With derive macros:

```toml
[dependencies]
byten = { version = "0.0", default-features = false, features = ["derive"] }
```

**Note:** When using derive macros in core-only mode, avoid using `Vec`, `Box`, `String`, or other allocation-dependent types in your structs.

#### With an allocator (core + alloc)

For `no_std` environments with an allocator:

```toml
[dependencies]
byten = { version = "0.0", default-features = false, features = ["alloc"] }
```

This adds support for:
- `Vec<T>` collections
- `Box<T>` heap allocation  
- Owned strings (`String`)
- Variable-length encoding (`UVarBECodec`)

With derive macros (recommended for most use cases):

```toml
[dependencies]
byten = { version = "0.0", default-features = false, features = ["alloc", "derive"] }
```

**Note:** The `derive` feature works in all modes (core-only, alloc, and std). The generated code will only use features that are enabled.

## Examples

The `byten/examples` directory contains several complete examples:

- **`array.rs`**: Encoding arrays with variable-length integers
- **`borrowed.rs`**: Zero-copy decoding with borrowed data and lifetimes
- **`archive.rs`**: Recursive structures (file system directory tree)
- **`icmp.rs`**: Network packet encoding (ICMP header)
- **`nostd.rs`**: Simple FS structures for `no_std` environments
- **`inline.rs`**: Using the inline `byten!()` macro for ad-hoc codecs

Run examples with:
```bash
cargo run --example borrowed
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please read our [Contributing Guidelines](CONTRIBUTING.md) for details on how to submit pull requests, report issues, and contribute to the project.

This project adheres to the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

