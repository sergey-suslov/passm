use anyhow::{anyhow, Ok, Result};
use log::debug;
use rust_crypto::pbkdf2::pbkdf2;
use rust_crypto::{
    aes, blockmodes,
    buffer::{self, BufferResult, ReadBuffer, WriteBuffer},
    hmac::Hmac,
    sha2::Sha256,
};

fn hash_master_password(master_password: String) -> [u8; 32] {
    let salt = [0u8; 16];
    // rng.fill_bytes(&mut salt);

    // 256-bit derived key
    let mut key = [0u8; 32];

    let mut mac = Hmac::new(Sha256::new(), master_password.as_bytes());

    pbkdf2(&mut mac, &salt[..], 256, &mut key);
    debug!(
        "{:?}",
        key.to_vec()
            .iter()
            .map(|e| format!("{:02X}", e))
            .collect::<String>()
    );
    key
}

pub fn export_private_key_bytes(plain: String, master_password: String) -> Result<Vec<u8>> {
    let armored = plain.as_bytes();
    debug!("Got armored key");
    let key = hash_master_password(master_password);
    debug!("Key:{:?}", key);
    Ok(encrypt(armored, &key, &[0u8; 16])?)
}

pub fn import_private_key_bytes(encripted: Vec<u8>, master_password: String) -> Result<Vec<u8>> {
    let key = hash_master_password(master_password);
    debug!("Key:{:?}", key);
    Ok(decrypt(&encripted, &key, &[0u8; 16])?)
}

fn encrypt(data: &[u8], key: &[u8], iv: &[u8]) -> Result<Vec<u8>> {
    let mut encryptor =
        aes::cbc_encryptor(aes::KeySize::KeySize256, key, iv, blockmodes::PkcsPadding);

    let mut final_result = Vec::<u8>::new();
    let mut read_buffer = buffer::RefReadBuffer::new(data);
    let mut buffer = [0; 4096];
    let mut write_buffer = buffer::RefWriteBuffer::new(&mut buffer);
    loop {
        let result = encryptor
            .encrypt(&mut read_buffer, &mut write_buffer, true)
            .map_err(|_| anyhow!("Error encrypting pass pgp secret"))?;
        final_result.extend(
            write_buffer
                .take_read_buffer()
                .take_remaining()
                .iter()
                .copied(),
        );

        match result {
            BufferResult::BufferUnderflow => break,
            BufferResult::BufferOverflow => {}
        }
    }

    Ok(final_result)
}

fn decrypt(encrypted_data: &[u8], key: &[u8], iv: &[u8]) -> Result<Vec<u8>> {
    let mut decryptor =
        aes::cbc_decryptor(aes::KeySize::KeySize256, key, iv, blockmodes::PkcsPadding);

    let mut final_result = Vec::<u8>::new();
    let mut read_buffer = buffer::RefReadBuffer::new(encrypted_data);
    let mut buffer = [0; 4096];
    let mut write_buffer = buffer::RefWriteBuffer::new(&mut buffer);

    loop {
        let result = decryptor
            .decrypt(&mut read_buffer, &mut write_buffer, true)
            .map_err(|e| anyhow!("Error decrypting pass pgp secret: {:?}", e))?;

        final_result.extend(
            write_buffer
                .take_read_buffer()
                .take_remaining()
                .iter()
                .copied(),
        );
        match result {
            BufferResult::BufferUnderflow => break,
            BufferResult::BufferOverflow => {}
        }
    }

    Ok(final_result)
}

mod tests {
    use simple_logger::SimpleLogger;

    use super::{export_private_key_bytes, import_private_key_bytes};

    #[test]
    fn export_pgp_key() {
        SimpleLogger::new()
            .with_level(log::LevelFilter::Debug)
            .init()
            .unwrap();

        let cyphertext = export_private_key_bytes("pgp".to_string(), "secret".to_string())
            .expect("error encrypting");
        let plain =
            import_private_key_bytes(cyphertext, "secret".to_string()).expect("error encrypting");
        assert_eq!(String::from_utf8(plain).unwrap(), "pgp".to_string());
        assert!(false);
    }
}
