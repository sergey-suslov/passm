use std::io::Read;

use anyhow::{Ok, Result};
use pgp::composed::{
    KeyType, SecretKey, SecretKeyParamsBuilder, SignedSecretKey, SubkeyParamsBuilder,
};
use pgp::crypto::{HashAlgorithm, PublicKeyAlgorithm, SymmetricKeyAlgorithm};
use pgp::packet::{self, SignatureConfig, SignatureConfigBuilder};
use pgp::types::{CompressionAlgorithm, KeyTrait, SecretKeyRepr, SecretKeyTrait};
use rand::thread_rng;
use rsa::{PaddingScheme, PublicKey};
use smallvec::smallvec;

pub struct Signer {
    sig_cfg: SignatureConfig,
    signing_key: SignedSecretKey,
    passphrase: Option<String>,
}

impl Signer {
    pub fn new(signing_key: SignedSecretKey, passphrase: Option<String>) -> Self {
        let now = chrono::Utc::now();
        let mut sig_cfg_bldr = SignatureConfigBuilder::default();
        let sig_cfg = sig_cfg_bldr
            .version(packet::SignatureVersion::V4)
            .typ(packet::SignatureType::Binary)
            .pub_alg(PublicKeyAlgorithm::RSA)
            .hash_alg(HashAlgorithm::SHA2_256)
            .issuer(Some(signing_key.key_id()))
            .created(Some(now))
            .unhashed_subpackets(vec![]) // must be initialized
            .hashed_subpackets(vec![
                packet::Subpacket::SignatureCreationTime(now),
                packet::Subpacket::Issuer(signing_key.key_id()),
            ]) // must be initialized
            .build()
            .unwrap();
        Self {
            sig_cfg,
            passphrase,
            signing_key,
        }
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
    use pgp::{
        types::{PublicKeyTrait, SecretKeyRepr, SecretKeyTrait},
        KeyType,
    };
    use rsa::{PaddingScheme, PublicKey};

    use super::Signer;

    #[test]
    fn generate_key() {
        let pass = String::from("pass");
        let plain = String::from("plain secret text");
        let secret_key = Signer::generate_key(KeyType::Rsa(2048), Some(pass.clone()));
        let signed_sk = secret_key.sign(|| pass.clone()).unwrap();
        let private_armored = signed_sk.to_armored_string(None).unwrap();

        let test_string_content = "My Test Data".to_owned();

        let signer = Signer::new(signed_sk, Some(pass));
        let encrypted: Vec<u8> = signer.encrypt(test_string_content.as_bytes()).unwrap();
        let decrypted = signer.decrypt(&encrypted).unwrap();

        assert_eq!(decrypted, test_string_content.as_bytes());
    }
}
