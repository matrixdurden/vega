//! Offline accounts. Microsoft sign-in will live next to this once the
//! Azure app is approved by Mojang.

use md5::{Digest, Md5};

pub struct Account {
    pub username: String,
    pub uuid: String,
    pub access_token: String,
    pub user_type: &'static str,
}

impl Account {
    pub fn offline(username: &str) -> Self {
        Self {
            username: username.to_owned(),
            uuid: offline_uuid(username),
            access_token: "0".to_owned(),
            user_type: "legacy",
        }
    }
}

/// Same UUID the vanilla server gives an offline player:
/// `UUID.nameUUIDFromBytes("OfflinePlayer:" + name)`, a version 3 UUID.
pub fn offline_uuid(username: &str) -> String {
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&Md5::digest(format!("OfflinePlayer:{username}").as_bytes()));
    bytes[6] = (bytes[6] & 0x0f) | 0x30;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    uuid::Uuid::from_bytes(bytes).simple().to_string()
}

/// Minecraft usernames: 3 to 16 letters, digits or underscores.
pub fn valid_username(username: &str) -> bool {
    (3..=16).contains(&username.len()) && username.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_java_offline_uuid() {
        // UUID.nameUUIDFromBytes("OfflinePlayer:Notch".getBytes(UTF_8))
        assert_eq!(offline_uuid("Notch"), "b50ad385829d3141a2167e7d7539ba7f");
    }

    #[test]
    fn usernames() {
        assert!(valid_username("Steve_42"));
        assert!(!valid_username("ab"));
        assert!(!valid_username("bad name"));
        assert!(!valid_username("şeyma"));
    }
}
