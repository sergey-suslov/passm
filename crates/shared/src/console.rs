use std::io::stdin;

use anyhow::{anyhow, Result};

pub fn read_passphrase(with_confirmation: bool) -> Result<String> {
    println!("Enter your passphrase:");
    let mut buffer = String::new();
    stdin().read_line(&mut buffer).unwrap();

    if with_confirmation {
        println!("Enter your passphrase again:");
        let mut buffer_repeat = String::new();
        stdin().read_line(&mut buffer_repeat).unwrap();
        println!();
        if buffer_repeat != buffer {
            return Err(anyhow!("Passphrases do not match"));
        }
    }
    Ok(buffer.trim_end().to_string())
}
