use aesencryption::{decrypt_hex, encrypt_hex};

fn main() {
    let plaintext = "6BC1BEE22E409F96E93D7E117393172AAE2D8A571E03AC9C9EB76FAC45AF8E5130C81C46A35CE411E5FBC1191A0A52EFF69F2445DF4F9B17AD2B417BE66C3710";
    let key = "2B7E151628AED2A6ABF7158809CF4F3C";

    let ciphertext = encrypt_hex(plaintext, key).expect("encryption should succeed");
    println!("Ciphertext: {ciphertext}");

    let recovered = decrypt_hex(&ciphertext, key).expect("decryption should succeed");
    println!("Recovered:  {recovered}");
}
