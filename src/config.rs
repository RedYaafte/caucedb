use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Profile {
    pub id: Uuid,
    pub name: String,
    pub engine: String,
    pub host: String,
    pub port: u16,
    pub address_type: String,
    pub service: String,
    pub descriptor: String,
    pub protocol: String,
    pub wallet: String,
    pub connect_timeout: u32,
    pub call_timeout: u32,
    pub row_limit: usize,
    pub auth: String,
    pub role: String,
    pub username: String,
    pub proxy_enabled: bool,
    pub proxy_username: String,
    pub save_password: bool,
    #[serde(skip)]
    pub password: String,
    #[serde(skip)]
    pub proxy_password: String,
}
impl Default for Profile {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "New connection".into(),
            engine: "Oracle".into(),
            host: "localhost".into(),
            port: 1521,
            address_type: "Service Name".into(),
            service: "FREEPDB1".into(),
            descriptor: String::new(),
            protocol: "TCP".into(),
            wallet: String::new(),
            connect_timeout: 10,
            call_timeout: 30,
            row_limit: 1000,
            auth: "Default".into(),
            role: "Default".into(),
            username: String::new(),
            proxy_enabled: false,
            proxy_username: String::new(),
            save_password: false,
            password: String::new(),
            proxy_password: String::new(),
        }
    }
}
impl Profile {
    pub fn validate(&self) -> Result<()> {
        let fail = |s: &str| Err(Error::Message(s.into()));
        if self.name.trim().is_empty() {
            return fail("Connection Name is required");
        }
        if self.engine != "Oracle" {
            return fail("Only Oracle is supported in this MVP");
        }
        if !["Default", "External"].contains(&self.auth.as_str()) {
            return fail("Unknown authentication type");
        }
        if !["Default", "SYSDBA", "SYSOPER"].contains(&self.role.as_str()) {
            return fail("Unknown role");
        }
        if !["Service Name", "SID", "TNS / Descriptor"].contains(&self.address_type.as_str()) {
            return fail("Unknown address type");
        }
        if !["TCP", "TCPS"].contains(&self.protocol.as_str()) {
            return fail("Protocol must be TCP or TCPS");
        }
        if self.auth == "Default" && self.username.trim().is_empty() {
            return fail("Username is required");
        }
        if self.proxy_enabled
            && (self.proxy_username.trim().is_empty()
                || self.username.trim().is_empty()
                || self.auth != "Default")
        {
            return fail(
                "Proxy authentication requires Default auth, proxy username and target username",
            );
        }
        if self.connect_timeout == 0 || self.call_timeout == 0 {
            return fail("Timeouts must be greater than zero");
        }
        if !(1..=100_000).contains(&self.row_limit) {
            return fail("Row limit must be between 1 and 100000");
        }
        if self.address_type == "TNS / Descriptor" {
            if self.descriptor.trim().is_empty() {
                return fail("TNS alias / descriptor is required");
            }
        } else {
            if self.host.trim().is_empty() || self.service.trim().is_empty() || self.port == 0 {
                return fail("Hostname, Port and Service Name / SID are required");
            }
            for value in [&self.host, &self.service, &self.wallet] {
                if value.contains(['(', ')', '\n', '\r', '"']) {
                    return fail("Connection fields contain invalid descriptor characters");
                }
            }
        }
        Ok(())
    }
    pub fn connect_string(&self) -> String {
        if self.address_type == "TNS / Descriptor" {
            return self.descriptor.trim().into();
        }
        let kind = if self.address_type == "SID" {
            "SID"
        } else {
            "SERVICE_NAME"
        };
        let security = if self.protocol == "TCPS" {
            let wallet = if self.wallet.is_empty() {
                String::new()
            } else {
                format!("(MY_WALLET_DIRECTORY=\"{}\")", self.wallet)
            };
            format!("(SECURITY=(SSL_SERVER_DN_MATCH=YES){wallet})")
        } else {
            String::new()
        };
        format!("(DESCRIPTION=(CONNECT_TIMEOUT={})(TRANSPORT_CONNECT_TIMEOUT={})(RETRY_COUNT=0)(ADDRESS=(PROTOCOL={})(HOST={})(PORT={}))(CONNECT_DATA=({}={})){})",
            self.connect_timeout, self.connect_timeout, self.protocol, self.host, self.port, kind, self.service, security)
    }
    pub fn credentials(&self) -> (String, String) {
        if self.proxy_enabled {
            (
                format!("{}[{}]", self.proxy_username, self.username),
                self.proxy_password.clone(),
            )
        } else {
            (self.username.clone(), self.password.clone())
        }
    }
    fn entry(&self, proxy: bool) -> Result<keyring::Entry> {
        keyring::Entry::new(
            "caucedb",
            &format!("{}:{}", self.id, if proxy { "proxy" } else { "user" }),
        )
        .map_err(|_| Error::Message("Secure credential store is unavailable".into()))
    }
    pub fn load_secrets(&mut self) -> Result<()> {
        if self.save_password {
            let secret = self
                .entry(self.proxy_enabled)?
                .get_password()
                .map_err(|_| {
                    Error::Message(
                        "Saved password unavailable; enter it again in the connection form".into(),
                    )
                })?;
            if self.proxy_enabled {
                self.proxy_password = secret;
            } else {
                self.password = secret;
            }
        }
        Ok(())
    }
    pub fn persist_secrets(&self) -> Result<()> {
        if self.save_password {
            let secret = if self.proxy_enabled {
                &self.proxy_password
            } else {
                &self.password
            };
            self.entry(self.proxy_enabled)?.set_password(secret).map_err(|_| Error::Message("Cannot save password: unlock/install your system keyring, or uncheck Save password".into()))?;
        }
        for proxy in [false, true] {
            if !self.save_password || proxy != self.proxy_enabled {
                // No entry is expected for profiles which have never saved a password.
                if let Ok(entry) = self.entry(proxy) {
                    match entry.delete_credential() {
                        Ok(()) | Err(keyring::Error::NoEntry) => {},
                        Err(_) => return Err(Error::Message("Could not remove old password from the keyring; unlock the keyring and retry".into())),
                    }
                }
            }
        }
        Ok(())
    }
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub connections: Vec<Profile>,
}
pub fn config_path() -> Result<PathBuf> {
    if let Some(path) = std::env::var_os("CAUCEDB_CONFIG") {
        return Ok(path.into());
    }
    directories::ProjectDirs::from("dev", "RedYaafte", "caucedb")
        .map(|d| d.config_dir().join("connections.toml"))
        .ok_or_else(|| {
            Error::Message("Cannot locate configuration directory; set CAUCEDB_CONFIG".into())
        })
}
impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        match fs::read_to_string(path) {
            Ok(s) => toml::from_str(&s)
                .map_err(|e| Error::Message(format!("Invalid configuration: {e}"))),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e.into()),
        }
    }
    pub fn save(&self, path: &Path) -> Result<()> {
        let text = toml::to_string_pretty(self).map_err(|e| Error::Message(e.to_string()))?;
        atomic_write(path, text.as_bytes())
    }
}
pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    use std::io::Write;
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent)?;
    let temp = parent.join(format!(".caucedb-{}.tmp", Uuid::new_v4()));
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let result = (|| -> Result<()> {
        let mut file = options.open(&temp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        fs::rename(&temp, path)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn secrets_never_serialize() {
        let p = Profile {
            password: "secret-normal".into(),
            proxy_password: "secret-proxy".into(),
            ..Profile::default()
        };
        let encoded = toml::to_string(&p).unwrap();
        assert!(!encoded.contains("secret-normal"));
        assert!(!encoded.contains("secret-proxy"));
    }
    #[test]
    fn descriptor_and_validation() {
        let mut p = Profile {
            username: "scott".into(),
            ..Profile::default()
        };
        p.validate().unwrap();
        assert!(p.connect_string().contains("SERVICE_NAME=FREEPDB1"));
        p.address_type = "SID".into();
        assert!(p.connect_string().contains("SID=FREEPDB1"));
        p.host = "host)(bad".into();
        assert!(p.validate().is_err());
    }
    #[test]
    fn config_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("connections.toml");
        let config = Config {
            connections: vec![Profile::default()],
        };
        config.save(&path).unwrap();
        assert_eq!(
            Config::load(&path).unwrap().connections[0].id,
            config.connections[0].id
        );
    }
}
