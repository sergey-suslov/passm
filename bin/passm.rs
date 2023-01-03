use core::app::App;
use std::{fs, process};

use anyhow::Result;
use config::config::Configuration;
use crypto::{pgp::KeyType, signer::Signer};
use simple_logger::SimpleLogger;

#[tokio::main]
async fn main() -> Result<()> {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .unwrap();
    let namespace_configuration = Configuration::init().unwrap();

    // Init signed secret key
    let ssk = match fs::metadata(namespace_configuration.private_key_path.clone()) {
        Ok(_) => {
            let file =
                fs::read_to_string(namespace_configuration.private_key_path.clone()).unwrap();
            Signer::parse_signed_secret_from_string(file)
                .expect("failed to parse key from the file")
        }
        Err(_) => {
            let passphrase = shared::console::read_passphrase(true).unwrap();
            println!("Starting key pair generation");
            let sk = Signer::generate_key(KeyType::Rsa(2048), Some(passphrase.clone()));
            println!("New key pair has been created!");

            let signed = Signer::sign_key(sk, Some(passphrase));
            fs::write(
                namespace_configuration.private_key_path,
                signed.to_armored_string(None).unwrap(),
            )
            .unwrap();
            signed
        }
    };

    let passphrase = shared::console::read_passphrase(false).unwrap();
    Signer::verify_key_passphrase(&ssk, Some(passphrase.clone())).unwrap_or_else(|_| {
        println!("Wrong passphrase");
        process::exit(1);
    });
    let signer = Signer::new(ssk, Some(passphrase));

    let mut app = App::new(signer, namespace_configuration.passwords_dir);
    app.run().await;

    Ok(())
}
