//! Mojang's library and argument rules.

use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    pub action: Action,
    #[serde(default)]
    pub os: Option<OsRule>,
    #[serde(default)]
    pub features: Option<HashMap<String, bool>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    Allow,
    Disallow,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OsRule {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub arch: Option<String>,
}

/// The OS name as Mojang spells it.
pub fn os_name() -> &'static str {
    if cfg!(windows) {
        "windows"
    } else if cfg!(target_os = "macos") {
        "osx"
    } else {
        "linux"
    }
}

/// No rules means allowed; otherwise start disallowed and let the last
/// matching rule decide. Optional features (demo mode, quick play, custom
/// resolution) are never enabled, so rules that require them never match.
pub fn allowed(rules: &[Rule]) -> bool {
    if rules.is_empty() {
        return true;
    }
    rules.iter().filter(|rule| matches(rule)).fold(false, |_, rule| rule.action == Action::Allow)
}

fn matches(rule: &Rule) -> bool {
    if let Some(os) = &rule.os {
        if os.name.as_deref().is_some_and(|name| name != os_name()) {
            return false;
        }
        if os.arch.as_deref().is_some_and(|arch| arch == "x86" && std::env::consts::ARCH != "x86") {
            return false;
        }
    }
    !rule.features.as_ref().is_some_and(|features| features.values().any(|&on| on))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules(json: &str) -> Vec<Rule> {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn empty_rules_allow() {
        assert!(allowed(&[]));
    }

    #[test]
    fn os_specific_rule() {
        let only_here = format!(r#"[{{"action":"allow","os":{{"name":"{}"}}}}]"#, os_name());
        assert!(allowed(&rules(&only_here)));
        assert!(!allowed(&rules(r#"[{"action":"allow","os":{"name":"plan9"}}]"#)));
        assert!(allowed(&rules(r#"[{"action":"allow"},{"action":"disallow","os":{"name":"plan9"}}]"#)));
    }

    #[test]
    fn feature_rules_never_match() {
        assert!(!allowed(&rules(r#"[{"action":"allow","features":{"is_demo_user":true}}]"#)));
    }
}
