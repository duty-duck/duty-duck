use std::{ops::Deref, str::FromStr};

use anyhow::anyhow;
use rusty_paseto::core::{Key, Local, PasetoSymmetricKey, V4};

pub struct SymetricEncryptionKey {
    value: PasetoSymmetricKey<V4, Local>,
}

impl SymetricEncryptionKey {
    #[cfg(test)]
    pub fn new_random() -> Self {
        let random_key = Key::try_new_random().unwrap();
        let value = PasetoSymmetricKey::from(random_key);
        Self { value }
    }
}

impl Deref for SymetricEncryptionKey {
    type Target = PasetoSymmetricKey<V4, Local>;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl FromStr for SymetricEncryptionKey {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let k = Key::try_from(s).map_err(|e| anyhow!("Failed to decode hex key: {e}"))?;
        let value = PasetoSymmetricKey::from(k);

        Ok(Self { value })
    }
}
