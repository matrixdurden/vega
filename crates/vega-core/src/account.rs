//! Accounts: offline names and Microsoft accounts.

use md5::{Digest, Md5};
use serde::{Deserialize, Serialize};

/// The identity a game is launched with.
pub struct Account {
    pub username: String,
    pub uuid: String,
    pub access_token: String,
    pub user_type: &'static str,
    pub xuid: Option<String>,
}

impl Account {
    pub fn offline(username: &str) -> Self {
        Self {
            username: username.to_owned(),
            uuid: offline_uuid(username),
            access_token: "0".to_owned(),
            user_type: "legacy",
            xuid: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AccountKind {
    Offline,
    Microsoft,
}

/// Tokens of a signed-in Microsoft account. Never sent to the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MicrosoftSession {
    pub refresh_token: String,
    pub access_token: String,
    /// Unix seconds when `access_token` stops working.
    pub expires_at: u64,
    #[serde(default)]
    pub xuid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredAccount {
    /// Minecraft UUID without dashes; also the account's id.
    pub id: String,
    pub kind: AccountKind,
    pub username: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub microsoft: Option<MicrosoftSession>,
}

/// accounts.json.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AccountStore {
    pub accounts: Vec<StoredAccount>,
    pub active: Option<String>,
}

impl AccountStore {
    pub fn active(&self) -> Option<&StoredAccount> {
        let id = self.active.as_deref();
        self.accounts.iter().find(|account| Some(account.id.as_str()) == id).or(self.accounts.first())
    }

    /// Adds or replaces an account (same id) and makes it the active one.
    pub fn upsert(&mut self, account: StoredAccount) {
        self.active = Some(account.id.clone());
        match self.accounts.iter_mut().find(|existing| existing.id == account.id) {
            Some(existing) => *existing = account,
            None => self.accounts.push(account),
        }
    }

    pub fn summaries(&self) -> Vec<AccountSummary> {
        let active = self.active().map(|account| account.id.clone());
        self.accounts
            .iter()
            .map(|account| AccountSummary {
                id: account.id.clone(),
                kind: account.kind,
                username: account.username.clone(),
                active: Some(&account.id) == active.as_ref(),
            })
            .collect()
    }
}

/// What the UI sees of an account.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountSummary {
    pub id: String,
    pub kind: AccountKind,
    pub username: String,
    pub active: bool,
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

    #[test]
    fn active_account_falls_back_to_first() {
        let offline = |name: &str| StoredAccount {
            id: offline_uuid(name),
            kind: AccountKind::Offline,
            username: name.into(),
            microsoft: None,
        };
        let mut store = AccountStore::default();
        assert!(store.active().is_none());
        store.upsert(offline("Alex"));
        store.upsert(offline("Steve"));
        assert_eq!(store.active().unwrap().username, "Steve");
        store.active = Some("gone".into());
        assert_eq!(store.active().unwrap().username, "Alex");
        store.upsert(offline("Steve"));
        assert_eq!(store.accounts.len(), 2);
        assert_eq!(store.summaries().iter().filter(|s| s.active).count(), 1);
    }
}
