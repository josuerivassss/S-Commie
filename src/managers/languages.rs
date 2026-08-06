use dashmap::DashMap;

pub struct LanguageCache {
    // alias (lowercased) -> (canonical language, version)
    entries: DashMap<String, (String, String)>,
}

impl LanguageCache {
    pub fn empty() -> Self {
        Self { entries: DashMap::new() }
    }

    /// Resolves a user-provided language/alias (case-insensitive) to its canonical
    /// Piston language name + version, or `None` if unsupported/cache not yet loaded.
    pub fn resolve(&self, alias: &str) -> Option<(String, String)> {
        self.entries.get(&alias.to_lowercase()).map(|e| e.value().clone())
    }

    pub fn is_loaded(&self) -> bool {
        !self.entries.is_empty()
    }

    pub fn all_aliases(&self) -> Vec<String> {
        let mut aliases: Vec<String> = self.entries.iter().map(|e| e.key().clone()).collect();
        aliases.sort();
        aliases
    }

    pub async fn refresh(&self, http: &reqwest::Client) -> anyhow::Result<()> {
        let runtimes: serde_json::Value = http.get("https://emkc.org/api/v2/piston/runtimes").send().await?.json().await?;
        let runtimes = runtimes.as_array().ok_or_else(|| anyhow::anyhow!("unexpected /runtimes response shape"))?;

        let mut fresh = std::collections::HashMap::new();
        for runtime in runtimes {
            let language = runtime.get("language").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let version = runtime.get("version").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            if language.is_empty() || version.is_empty() {
                continue;
            }
            fresh.insert(language.to_lowercase(), (language.clone(), version.clone()));
            if let Some(aliases) = runtime.get("aliases").and_then(|v| v.as_array()) {
                for alias in aliases.iter().filter_map(|a| a.as_str()) {
                    fresh.insert(alias.to_lowercase(), (language.clone(), version.clone()));
                }
            }
        }

        if fresh.is_empty() {
            anyhow::bail!("refresh produced zero languages, keeping previous cache");
        }

        self.entries.clear();
        for (k, v) in fresh {
            self.entries.insert(k, v);
        }
        Ok(())
    }
}