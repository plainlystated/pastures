use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Everything tunable. Defaults are compiled in; a config file is only needed to change them.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub warmth: WarmthConfig,
    pub paths: PathsConfig,
}

/// `warmth = turns^investment_exponent / (hours_since_your_last_turn + staleness_floor_hours)^staleness_exponent`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WarmthConfig {
    /// How much a turn counts. Above 1 favours deep sessions; below 1 flattens them.
    pub investment_exponent: f64,
    /// How fast a session cools. Above 1 makes the list more recency-like.
    pub staleness_exponent: f64,
    /// Added to staleness before dividing, so a session touched seconds ago has a finite score.
    pub staleness_floor_hours: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PathsConfig {
    /// Claude Code's home. Defaults to `$CLAUDE_CONFIG_DIR` or `~/.claude`.
    pub claude_home: Option<PathBuf>,
}

impl Default for WarmthConfig {
    fn default() -> Self {
        Self {
            investment_exponent: 1.0,
            staleness_exponent: 1.0,
            staleness_floor_hours: 0.25,
        }
    }
}

impl Config {
    /// The conventional location: `~/.config/pastures/config.toml`.
    pub fn default_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("pastures").join("config.toml"))
    }

    /// Loads the file at `path` if it exists, otherwise returns defaults.
    pub fn load(path: Option<&Path>) -> Result<Self> {
        let path = match path {
            Some(p) => p.to_path_buf(),
            None => match Self::default_path() {
                Some(p) => p,
                None => return Ok(Self::default()),
            },
        };
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        toml::from_str(&text).with_context(|| format!("parsing {}", path.display()))
    }

    /// Resolved Claude home: config, then `$CLAUDE_CONFIG_DIR`, then `~/.claude`.
    pub fn claude_home(&self) -> Result<PathBuf> {
        if let Some(p) = &self.paths.claude_home {
            return Ok(expand_tilde(p));
        }
        if let Some(p) = std::env::var_os("CLAUDE_CONFIG_DIR") {
            return Ok(PathBuf::from(p));
        }
        dirs::home_dir()
            .map(|h| h.join(".claude"))
            .context("could not determine home directory")
    }

    pub fn to_toml(&self) -> String {
        toml::to_string_pretty(self).expect("config serialises")
    }
}

fn expand_tilde(p: &Path) -> PathBuf {
    if let Ok(rest) = p.strip_prefix("~") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    }
    p.to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_round_trip_through_toml() {
        let text = Config::default().to_toml();
        let back: Config = toml::from_str(&text).unwrap();
        assert_eq!(back.warmth.staleness_floor_hours, 0.25);
        assert!(back.paths.claude_home.is_none());
    }

    #[test]
    fn partial_file_keeps_other_defaults() {
        let c: Config = toml::from_str("[warmth]\nstaleness_exponent = 2.0\n").unwrap();
        assert_eq!(c.warmth.staleness_exponent, 2.0);
        assert_eq!(c.warmth.investment_exponent, 1.0);
    }
}
