use byten::{Decode, DecodeOwned, DefaultCodec, Encode, EncodeToVec as _, Measure};

#[derive(Debug, DefaultCodec, Encode, Measure, DecodeOwned)]
pub struct Foo {
    #[byten(u16 $uvarbe $arr)]
    pub bar: [u16; 4],
}

fn main() {
    let value = Foo {
        bar: [1, 2, 3, 255],
    };

    let encoded = value.encode_to_vec().unwrap();
    println!("Encoded Foo: {:?}", encoded);

    let mut offset = 0;
    let decoded = Foo::decode(&encoded, &mut offset).unwrap();
    println!("Decoded Foo: {:?}", decoded);
}
