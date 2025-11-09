use byten::{DecodeOwned, DefaultCodec, Encode, Measure, MeasureFixed};

#[derive(Debug, PartialEq, Eq, DefaultCodec, DecodeOwned, Encode, Measure)]
struct Person {
    #[byten($be)]
    id: u32,
    #[byten( $bytes[u8] $utf8 $own )]
    name: String,
    birthday: Date,
    #[byten( Color for[usize $uvarbe] )]
    favorite_colors: Vec<Color>,
}

const fn custom_codec() -> ::byten::EndianCodec<u16> {
    ::byten::EndianCodec::new(::byten::Endianness::Big)
}

#[derive(Debug, PartialEq, Eq, DefaultCodec, DecodeOwned, Encode, MeasureFixed)]
struct Date {
    day: u8,
    month: u8,
    #[byten({ custom_codec() })]
    year: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, DefaultCodec, DecodeOwned, Encode, Measure)]
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
    Code(#[byten($be)] u64) = 7,
    Name(
        #[byten( $bytes[=4] )]
        #[byten( $utf8 $own )]
        String,
    ) = 8,
    Unknown() = 255,
}

#[cfg(test)]
mod test {
    use byten::{Decode as _, EncodeToVec as _};

    use super::*;

    #[test]
    fn test_person_codec() {
        let person = Person {
            id: 123456,
            name: "Alice".to_string(),
            birthday: Date {
                day: 23,
                month: 10,
                year: 1965,
            },
            favorite_colors: vec![
                Color::Red,
                Color::Grayscale(128),
                Color::RGBa {
                    red: 255,
                    green: 0,
                    blue: 0,
                    alpha: 65535,
                },
                Color::Gradient(Box::new(Color::Green), Box::new(Color::Blue)),
                Color::Code(0b110010101010),
                Color::Name("Cyan".into()),
                Color::Unknown(),
            ],
        };

        let expected_encoded = vec![
            0x00, 0x01, 0xe2, 0x40, // id: U32BE(123456)
            0x05, // name length: var::USizeBE(5)
            0x41, 0x6c, 0x69, 0x63, 0x65, // name: "Alice"
            23,   // birthday.day: u8(23)
            10,   // birthday.month: u8(10)
            0x07, 0xAD,       // birthday.year: U16BE(1965)
            0b00000111, // favorite_colors length: var::USizeBE(7)
            0x01, 0x00, // Color::Red discriminant: U16LE(1)
            0x04, 0x00, // Color::Grayscale discriminant: U16LE(4)
            0x00, 0x80, // Grayscale value: U16BE(128)
            0x05, 0x00, // Color::RGBa discriminant: U16LE(5)
            0xff, 0x00, 0x00, // RGB values
            0xff, 0xff, // alpha: U16BE(65535)
            0x06, 0x00, // Color::Gradient discriminant: U16LE(6)
            0x02, 0x00, // Color::Green discriminant: U16LE(2)
            0x03, 0x00, // Color::Blue discriminant: U16LE(3)
            0x07, 0x00, // Color::Code discriminant: U16LE(7);
            // Color::Code value: U64BE(0b110010101010)
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // padding for U64BE
            0b1100, 0b10101010, // full bytes
            0x08, 0x00, // Color::Name discriminant: U16LE(8)
            0x43, 0x79, 0x61, 0x6e, // name: "Cyan"
            0xff, 0x00, // Color::Unknown discriminant: U16LE(255)
        ];

        let size = person.measure().expect("Measuring failed");
        assert_eq!(size, expected_encoded.len());

        let encoded = person.encode_to_vec().expect("Encoding failed");
        assert_eq!(encoded, expected_encoded);
        let decoded = Person::decode(&encoded, &mut 0).expect("Decoding failed");
        assert_eq!(person, decoded);
    }
}
