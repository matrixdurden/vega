//! Microsoft sign-in: device code login, then Xbox Live, XSTS and the
//! Minecraft services, as described on https://minecraft.wiki/w/Microsoft_authentication.
//!
//! It needs the client id of an Azure app with "Allow public client flows"
//! turned on and approved by Mojang for the Minecraft API. Set it with the
//! `VEGA_MSA_CLIENT_ID` environment variable, at build time or at run time.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{
    Error, Result,
    account::{AccountKind, MicrosoftSession, StoredAccount},
    net::Http,
};

const DEVICE_CODE_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode";
const TOKEN_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";
const XBL_URL: &str = "https://user.auth.xboxlive.com/user/authenticate";
const XSTS_URL: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";
const MC_LOGIN_URL: &str = "https://api.minecraftservices.com/authentication/login_with_xbox";
const MC_PROFILE_URL: &str = "https://api.minecraftservices.com/minecraft/profile";
const SCOPE: &str = "XboxLive.signin offline_access";

/// The Azure app id, if this build or environment has one.
pub fn client_id() -> Option<String> {
    std::env::var("VEGA_MSA_CLIENT_ID")
        .ok()
        .or_else(|| option_env!("VEGA_MSA_CLIENT_ID").map(String::from))
        .filter(|id| !id.trim().is_empty())
}

fn require_client_id() -> Result<String> {
    client_id().ok_or_else(|| Error::other("Microsoft sign-in is not set up in this build (VEGA_MSA_CLIENT_ID)"))
}

/// What the player needs to finish signing in on microsoft.com/link.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCode {
    #[serde(rename(deserialize = "user_code"))]
    pub user_code: String,
    #[serde(rename(deserialize = "verification_uri"))]
    pub verification_uri: String,
    #[serde(rename(deserialize = "expires_in"))]
    pub expires_in: u64,
    #[serde(skip_serializing, rename(deserialize = "device_code"))]
    pub device_code: String,
    #[serde(skip_serializing, default = "default_interval")]
    pub interval: u64,
}

fn default_interval() -> u64 {
    5
}

#[derive(Deserialize)]
struct MsToken {
    access_token: String,
    refresh_token: String,
}

pub async fn start_device_login(http: &Http) -> Result<DeviceCode> {
    let client_id = require_client_id()?;
    let response =
        http.client().post(DEVICE_CODE_URL).form(&[("client_id", client_id.as_str()), ("scope", SCOPE)]).send().await?;
    let status = response.status();
    let body: Value = response.json().await?;
    if !status.is_success() {
        return Err(Error::other(format!("Microsoft refused the sign-in request: {}", ms_error(&body))));
    }
    Ok(serde_json::from_value(body)?)
}

/// Waits until the player has entered the code, then finishes the sign-in.
pub async fn finish_device_login(http: &Http, code: &DeviceCode) -> Result<StoredAccount> {
    let client_id = require_client_id()?;
    let deadline = now() + code.expires_in;
    let mut interval = code.interval.max(1);
    let token = loop {
        tokio::time::sleep(Duration::from_secs(interval)).await;
        if now() > deadline {
            return Err(Error::other("the sign-in code expired, try again"));
        }
        let response = http
            .client()
            .post(TOKEN_URL)
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ("client_id", client_id.as_str()),
                ("device_code", code.device_code.as_str()),
            ])
            .send()
            .await?;
        if response.status().is_success() {
            break response.json::<MsToken>().await?;
        }
        let body: Value = response.json().await?;
        match body["error"].as_str() {
            Some("authorization_pending") => {}
            Some("slow_down") => interval += 5,
            Some("authorization_declined") => return Err(Error::other("sign-in was cancelled")),
            Some("expired_token") => return Err(Error::other("the sign-in code expired, try again")),
            _ => return Err(Error::other(format!("Microsoft sign-in failed: {}", ms_error(&body)))),
        }
    };
    minecraft_account(http, token).await
}

/// Renews a Microsoft account's Minecraft token when it is about to expire.
pub async fn refresh(http: &Http, account: &StoredAccount) -> Result<StoredAccount> {
    let session = account.microsoft.as_ref().ok_or_else(|| Error::other("not a Microsoft account"))?;
    if session.expires_at > now() + 300 {
        return Ok(account.clone());
    }
    let client_id = require_client_id()?;
    let response = http
        .client()
        .post(TOKEN_URL)
        .form(&[
            ("grant_type", "refresh_token"),
            ("client_id", client_id.as_str()),
            ("refresh_token", session.refresh_token.as_str()),
            ("scope", SCOPE),
        ])
        .send()
        .await?;
    if !response.status().is_success() {
        let body: Value = response.json().await.unwrap_or_default();
        return Err(Error::other(format!("sign in to {} again: {}", account.username, ms_error(&body))));
    }
    minecraft_account(http, response.json().await?).await
}

/// Microsoft token → Xbox Live → XSTS → Minecraft token and profile.
async fn minecraft_account(http: &Http, token: MsToken) -> Result<StoredAccount> {
    let xbl = post_json(
        http,
        XBL_URL,
        &json!({
            "Properties": { "AuthMethod": "RPS", "SiteName": "user.auth.xboxlive.com", "RpsTicket": format!("d={}", token.access_token) },
            "RelyingParty": "http://auth.xboxlive.com",
            "TokenType": "JWT",
        }),
    )
    .await?;
    let xbl_token = xbl["Token"].as_str().ok_or_else(|| Error::other("Xbox Live returned no token"))?;

    let response = http
        .client()
        .post(XSTS_URL)
        .json(&json!({
            "Properties": { "SandboxId": "RETAIL", "UserTokens": [xbl_token] },
            "RelyingParty": "rp://api.minecraftservices.com/",
            "TokenType": "JWT",
        }))
        .send()
        .await?;
    let status = response.status();
    let xsts: Value = response.json().await.unwrap_or_default();
    if !status.is_success() {
        return Err(Error::other(xsts_error(xsts["XErr"].as_u64())));
    }
    let xsts_token = xsts["Token"].as_str().ok_or_else(|| Error::other("Xbox Live returned no token"))?;
    let claims = &xsts["DisplayClaims"]["xui"][0];
    let uhs = claims["uhs"].as_str().ok_or_else(|| Error::other("Xbox Live returned no user hash"))?;
    let xuid = claims["xid"].as_str().map(String::from);

    let response = http
        .client()
        .post(MC_LOGIN_URL)
        .json(&json!({ "identityToken": format!("XBL3.0 x={uhs};{xsts_token}") }))
        .send()
        .await?;
    match response.status().as_u16() {
        200 => {}
        403 => return Err(Error::other("Mojang has not approved this build's Azure app for Minecraft yet")),
        code => return Err(Error::other(format!("Minecraft login failed ({code})"))),
    }
    let login: Value = response.json().await?;
    let access_token = login["access_token"].as_str().ok_or_else(|| Error::other("Minecraft returned no token"))?;
    let expires_in = login["expires_in"].as_u64().unwrap_or(86_400);

    let response = http.client().get(MC_PROFILE_URL).bearer_auth(access_token).send().await?;
    if response.status().as_u16() == 404 {
        return Err(Error::other("this Microsoft account does not own Minecraft: Java Edition"));
    }
    let profile: Value = response.error_for_status()?.json().await?;
    let (Some(id), Some(name)) = (profile["id"].as_str(), profile["name"].as_str()) else {
        return Err(Error::other("Minecraft returned an incomplete profile"));
    };

    Ok(StoredAccount {
        id: id.to_owned(),
        kind: AccountKind::Microsoft,
        username: name.to_owned(),
        microsoft: Some(MicrosoftSession {
            refresh_token: token.refresh_token,
            access_token: access_token.to_owned(),
            expires_at: now() + expires_in,
            xuid,
        }),
    })
}

async fn post_json(http: &Http, url: &str, body: &Value) -> Result<Value> {
    let response = http.client().post(url).header("Accept", "application/json").json(body).send().await?;
    Ok(response.error_for_status()?.json().await?)
}

fn ms_error(body: &Value) -> String {
    body["error_description"].as_str().or(body["error"].as_str()).unwrap_or("unknown error").to_owned()
}

fn xsts_error(code: Option<u64>) -> String {
    match code {
        Some(2148916233) => "this Microsoft account has no Xbox profile yet; sign in once on xbox.com".into(),
        Some(2148916235) => "Xbox Live is not available in this account's country".into(),
        Some(2148916236 | 2148916237) => "this account needs adult verification on xbox.com".into(),
        Some(2148916238) => "child accounts must be added to a Family by an adult first".into(),
        Some(code) => format!("Xbox Live refused the sign-in ({code})"),
        None => "Xbox Live refused the sign-in".into(),
    }
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_device_code_response() {
        let code: DeviceCode = serde_json::from_value(json!({
            "user_code": "ABCD-EFGH",
            "device_code": "secret",
            "verification_uri": "https://www.microsoft.com/link",
            "expires_in": 900,
            "interval": 5,
            "message": "..."
        }))
        .unwrap();
        assert_eq!(code.user_code, "ABCD-EFGH");
        // The device code stays in the backend.
        let sent = serde_json::to_value(&code).unwrap();
        assert_eq!(
            sent,
            json!({ "userCode": "ABCD-EFGH", "verificationUri": "https://www.microsoft.com/link", "expiresIn": 900 })
        );
    }

    #[test]
    fn explains_xsts_errors() {
        assert!(xsts_error(Some(2148916238)).contains("Family"));
    }
}
