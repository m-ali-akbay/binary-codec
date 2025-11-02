use std::ffi::{CStr, CString};

use byten::{Decode, DecodeOwned, DefaultCodec, Encode, Measure, EncodeToVec as _};

#[derive(Debug, DefaultCodec, Encode, Measure, DecodeOwned)]
pub struct Directory {
    #[byten(CStr $own)]
    pub name: CString,
    #[byten($vec(Box<Entry>)[u16 $be])]
    pub entries: Vec<Box<Entry>>,
}

#[derive(Debug, DefaultCodec, Encode, Measure, DecodeOwned)]
pub struct File {
    #[byten(CStr $own)]
    pub name: CString,
    #[byten($bytes[u16 $be] $own)]
    pub content: Vec<u8>,
    #[byten(u32 $be $opt)]
    pub assigned_application_id: Option<u32>,
}

#[derive(Debug, DefaultCodec, Encode, Measure, DecodeOwned)]
#[repr(u8)]
pub enum Entry {
    File(File) = 1,
    Directory(Directory) = 2,
}

fn main() {
    let dir = Directory {
        name: CString::new("root").unwrap(),
        entries: vec![
            Box::new(Entry::File(File {
                name: CString::new("file1.txt").unwrap(),
                content: b"Hello, World!".to_vec(),
                assigned_application_id: Some(42),
            })),
            Box::new(Entry::Directory(Directory {
                name: CString::new("subdir").unwrap(),
                entries: vec![
                    Box::new(Entry::File(File {
                        name: CString::new("file2.txt").unwrap(),
                        content: b"Rust is awesome!".to_vec(),
                        assigned_application_id: None,
                    })),
                ],
            })),
        ],
    };

    let encoded = dir.encode_to_vec().unwrap();
    println!("Encoded Directory: {:?}", encoded);

    let mut offset = 0;
    let decoded_dir = Directory::decode(&encoded, &mut offset).unwrap();
    println!("Decoded Directory: {:?}", decoded_dir);
}
