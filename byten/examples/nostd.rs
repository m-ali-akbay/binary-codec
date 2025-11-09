#![no_std]
extern crate alloc;

use byten::{Decode, DecodeOwned, DefaultCodec, Encode, EncodeToVec as _, Measure};

#[derive(Debug, DefaultCodec, Encode, Measure, DecodeOwned, PartialEq, Eq)]
pub struct Directory {
    pub name: heapless::CString<32>,

    #[byten(Entry box for[u16 $be])]
    pub entries: heapless::Vec<alloc::boxed::Box<Entry>, 16>,
}

#[derive(Debug, DefaultCodec, Encode, Measure, DecodeOwned, PartialEq, Eq)]
pub struct File {
    pub name: heapless::CString<32>,

    #[byten(u8 for[u16 $be])]
    pub content: heapless::Vec<u8, 256>,

    #[byten(u32 $be ?)]
    pub assigned_application_id: Option<u32>,
}

#[derive(Debug, DefaultCodec, Encode, Measure, DecodeOwned, PartialEq, Eq)]
#[repr(u8)]
pub enum Entry {
    File(File) = 1,
    Directory(Directory) = 2,
}

fn main() {
    let dir = Directory {
        name: heapless::CString::from_bytes_with_nul(b"root\0").unwrap(),
        entries: heapless::Vec::from_array([
            alloc::boxed::Box::new(Entry::File(File {
                name: heapless::CString::from_bytes_with_nul(b"file1.txt\0").unwrap(),
                content: heapless::Vec::from_slice(b"Hello, World!").unwrap(),
                assigned_application_id: Some(42),
            })),
            alloc::boxed::Box::new(Entry::Directory(Directory {
                name: heapless::CString::from_bytes_with_nul(b"subdir\0").unwrap(),
                entries: heapless::Vec::from_array([alloc::boxed::Box::new(Entry::File(File {
                    name: heapless::CString::from_bytes_with_nul(b"file2.txt\0").unwrap(),
                    content: heapless::Vec::from_slice(b"Rust is awesome!").unwrap(),
                    assigned_application_id: None,
                }))]),
            })),
        ]),
    };

    let encoded = dir.encode_to_vec().unwrap();

    let mut offset = 0;
    let decoded_dir = Directory::decode(&encoded, &mut offset).unwrap();

    assert!(dir == decoded_dir);
}
