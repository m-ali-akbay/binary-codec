use std::ffi::CStr;

use byten::{Decode, DefaultCodec, Encode, EncodeToVec as _, Measure};

#[derive(Debug, DefaultCodec, Encode, Decode, Measure)]
pub struct Person<'encoded> {
    pub first_name: &'encoded CStr,

    pub last_name: &'encoded CStr,

    #[byten($bytes[u16 $be] $utf8)]
    pub address: &'encoded str,

    #[byten($bytes[u32 $uvarbe])]
    pub avatar_image: &'encoded [u8],

    passcode: &'encoded [u8; 4],

    #[byten(.. $utf8)]
    pub extra_data: &'encoded str,
}

fn main() {
    let person = Person {
        first_name: CStr::from_bytes_with_nul(b"Alice\0").unwrap(),
        last_name: CStr::from_bytes_with_nul(b"Smith\0").unwrap(),
        address: "123 Main St, Springfield",
        avatar_image: &[0x11, 0x22, 0x33, 0x44, 0x55, 0x66],
        passcode: &[1, 2, 3, 4],
        extra_data: &"Some extra information",
    };

    let encoded = person.encode_to_vec().unwrap();
    println!("Encoded Person: {:?}", encoded);

    let mut offset = 0;
    let decoded_person = Person::decode(&encoded, &mut offset).unwrap();
    println!("Decoded Person: {:?}", decoded_person);
}
