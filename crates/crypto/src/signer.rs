use anyhow::{Ok, Result};
use pgp::composed::{
    KeyType, SecretKey, SecretKeyParamsBuilder, SignedSecretKey, SubkeyParamsBuilder,
};
use pgp::crypto::{HashAlgorithm, PublicKeyAlgorithm, SymmetricKeyAlgorithm};
use pgp::packet::{self, SignatureConfig, SignatureConfigBuilder};
use pgp::types::{CompressionAlgorithm, KeyTrait, SecretKeyRepr, SecretKeyTrait};
use pgp::Deserializable;
use rand::thread_rng;
use rsa::{PaddingScheme, PublicKey};
use smallvec::smallvec;

pub struct Signer {
    signing_key: SignedSecretKey,
    passphrase: Option<String>,
}

impl Signer {
    pub fn new(signing_key: SignedSecretKey, passphrase: Option<String>) -> Self {
        Self {
            passphrase,
            signing_key,
        }
    }

    pub fn verify_key_passphrase(
        ssk: &SignedSecretKey,
        passphrase: Option<String>,
    ) -> Result<(), pgp::errors::Error> {
        ssk.unlock(
            || passphrase.unwrap_or_else(|| "".to_owned()),
            |_| std::result::Result::Ok(()),
        )
    }

    pub fn parse_signed_secret_from_string(key: String) -> Result<SignedSecretKey> {
        let (secret, _header) = SignedSecretKey::from_string(&key)?;
        secret.verify()?;
        Ok(secret)
    }

    pub fn generate_key(kt: KeyType, passphrase: Option<String>) -> SecretKey {
        let key_params = SecretKeyParamsBuilder::default()
            .key_type(kt)
            .can_create_certificates(true)
            .can_sign(true)
            .primary_user_id("Me <me@mail.com>".into())
            .preferred_symmetric_algorithms(smallvec![
                SymmetricKeyAlgorithm::AES256,
                SymmetricKeyAlgorithm::AES192,
                SymmetricKeyAlgorithm::AES128,
            ])
            .preferred_hash_algorithms(smallvec![
                HashAlgorithm::SHA2_256,
                HashAlgorithm::SHA2_384,
                HashAlgorithm::SHA2_512,
                HashAlgorithm::SHA2_224,
                HashAlgorithm::SHA1,
            ])
            .preferred_compression_algorithms(smallvec![
                CompressionAlgorithm::ZLIB,
                CompressionAlgorithm::ZIP,
            ])
            .passphrase(passphrase)
            .build()
            .unwrap();
        key_params
            .generate()
            .expect("failed to generate secret key, encrypted")
    }

    pub fn sign_key(sk: SecretKey, passphrase: Option<String>) -> SignedSecretKey {
        sk.sign(|| passphrase.unwrap_or_else(|| "".to_string()))
            .unwrap()
    }

    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut decrypted: Vec<u8> = vec![];

        self.signing_key
            .unlock(
                || self.passphrase.clone().unwrap_or_else(|| "".to_owned()),
                |unlocked| {
                    match unlocked {
                        SecretKeyRepr::RSA(k) => {
                            let plain = k
                                .decrypt(PaddingScheme::new_pkcs1v15_encrypt(), data)
                                .expect("failed to decrypt");
                            decrypted = plain;
                        }
                        _ => panic!("unexpected params type {:?}", unlocked),
                    }
                    std::result::Result::Ok(())
                },
            )
            .unwrap();
        Ok(decrypted)
    }
    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut encrypted: Vec<u8> = vec![];

        self.signing_key
            .unlock(
                || self.passphrase.clone().unwrap_or_else(|| "".to_owned()),
                |unlocked| {
                    let mut rng = thread_rng();
                    match unlocked {
                        SecretKeyRepr::RSA(k) => {
                            encrypted = {
                                let k: rsa::RsaPrivateKey = k.clone();
                                let pk: rsa::RsaPublicKey = k.into();
                                pk.encrypt(&mut rng, PaddingScheme::new_pkcs1v15_encrypt(), data)
                                    .expect("failed to encrypt")
                            };
                        }
                        _ => panic!("unexpected params type {:?}", unlocked),
                    }
                    std::result::Result::Ok(())
                },
            )
            .unwrap();
        Ok(encrypted)
    }
}

#[cfg(test)]
mod tests {
    use log::{debug, info};
    use pgp::KeyType;

    use super::Signer;

    #[test]
    fn generate_key() {
        let _ = simple_logger::init().unwrap();
        let pass = String::from("pass");
        debug!("Generating key");
        let secret_key = Signer::generate_key(KeyType::Rsa(2048), Some(pass.clone()));
        debug!("Key generated");
        let signed_sk = secret_key.sign(|| pass.clone()).unwrap();
        let private_armored_string = signed_sk.to_armored_string(None).unwrap();
        info!("Private armored:{}", private_armored_string);

        let parsed_signed_sk =
            Signer::parse_signed_secret_from_string(private_armored_string.clone()).unwrap();
        assert_eq!(
            parsed_signed_sk.to_armored_bytes(None).unwrap(),
            signed_sk.to_armored_bytes(None).unwrap()
        );

        let test_string_content = "My Test Data".to_owned();

        let signer = Signer::new(signed_sk, Some(pass));
        let encrypted: Vec<u8> = signer.encrypt(test_string_content.as_bytes()).unwrap();
        let decrypted = signer.decrypt(&encrypted).unwrap();

        assert_eq!(decrypted, test_string_content.as_bytes());
        assert!(false);
    }
}
