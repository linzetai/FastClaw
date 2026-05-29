use aes::cipher::{BlockDecrypt, BlockEncrypt, KeyInit};
use aes::Aes128;

/// AES-128-ECB encrypt with PKCS7 padding.
pub fn aes128_ecb_encrypt(plaintext: &[u8], key: &[u8; 16]) -> Vec<u8> {
    let cipher = Aes128::new(key.into());
    let padded = pkcs7_pad(plaintext, 16);
    let mut output = padded;
    for chunk in output.chunks_exact_mut(16) {
        let block = aes::Block::from_mut_slice(chunk);
        cipher.encrypt_block(block);
    }
    output
}

/// AES-128-ECB decrypt and remove PKCS7 padding.
pub fn aes128_ecb_decrypt(ciphertext: &[u8], key: &[u8; 16]) -> anyhow::Result<Vec<u8>> {
    if !ciphertext.len().is_multiple_of(16) {
        anyhow::bail!("ciphertext length not a multiple of 16");
    }
    let cipher = Aes128::new(key.into());
    let mut output = ciphertext.to_vec();
    for chunk in output.chunks_exact_mut(16) {
        let block = aes::Block::from_mut_slice(chunk);
        cipher.decrypt_block(block);
    }
    pkcs7_unpad(&output)
}

fn pkcs7_pad(data: &[u8], block_size: usize) -> Vec<u8> {
    let padding_len = block_size - (data.len() % block_size);
    let mut padded = Vec::with_capacity(data.len() + padding_len);
    padded.extend_from_slice(data);
    padded.extend(std::iter::repeat_n(padding_len as u8, padding_len));
    padded
}

fn pkcs7_unpad(data: &[u8]) -> anyhow::Result<Vec<u8>> {
    if data.is_empty() {
        anyhow::bail!("empty data");
    }
    let pad_byte = *data.last().unwrap();
    let pad_len = pad_byte as usize;
    if pad_len == 0 || pad_len > 16 || pad_len > data.len() {
        anyhow::bail!("invalid PKCS7 padding");
    }
    if !data[data.len() - pad_len..].iter().all(|&b| b == pad_byte) {
        anyhow::bail!("corrupted PKCS7 padding");
    }
    Ok(data[..data.len() - pad_len].to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip() {
        let key = [0x42u8; 16];
        let plain = b"Hello, WeChat AES-128-ECB!";
        let encrypted = aes128_ecb_encrypt(plain, &key);
        let decrypted = aes128_ecb_decrypt(&encrypted, &key).unwrap();
        assert_eq!(&decrypted, plain);
    }

    #[test]
    fn test_roundtrip_block_aligned() {
        let key = [0xABu8; 16];
        let plain = [0x00u8; 32]; // exactly 2 blocks
        let encrypted = aes128_ecb_encrypt(&plain, &key);
        assert_eq!(encrypted.len(), 48); // 2 blocks + 1 padding block
        let decrypted = aes128_ecb_decrypt(&encrypted, &key).unwrap();
        assert_eq!(&decrypted, &plain);
    }

    #[test]
    fn test_empty() {
        let key = [0x01u8; 16];
        let encrypted = aes128_ecb_encrypt(b"", &key);
        assert_eq!(encrypted.len(), 16); // one full padding block
        let decrypted = aes128_ecb_decrypt(&encrypted, &key).unwrap();
        assert!(decrypted.is_empty());
    }
}
