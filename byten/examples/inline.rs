use byten::{byten, Decoder, EncoderToVec as _};

fn main() {
    let codec = byten!( $bytes[u32 $be] $utf8 );

    let original_str = "Hello, Byten!";
    let encoded = codec.encode_to_vec(original_str).unwrap();
    println!("Encoded bytes: {:?}", encoded);

    let decoded_str = codec.decode(&encoded, &mut 0).unwrap();
    println!("Decoded string: {}", decoded_str);
}
