use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// Embedded default price table (version 2026-09-11.v2).
pub const EMBEDDED_PRICE_TABLE_JSON: &str =
    include_str!("../data/price_table_2026-09-11.v2.json");

pub const DEFAULT_PRICE_TABLE_VERSION: &str = "2026-09-11.v2";

#[derive(Debug, Error)]
pub enum PriceTableError {
    #[error("invalid price table JSON: {0}")]
    Json(#[from] serde_json::Error),
}

/// Per-model rates in USD per 1_000_000 tokens.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelRates {
    pub input: f64,
    pub output: f64,
    #[serde(default)]
    pub cache_read: Option<f64>,
    /// Prefer 5m cache-write bucket when only one write rate is present.
    #[serde(default)]
    pub cache_write_5m: Option<f64>,
    #[serde(default)]
    pub cache_write_1h: Option<f64>,
    #[serde(default)]
    pub alias_of: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
}

impl ModelRates {
    pub fn cache_write_rate(&self) -> f64 {
        self.cache_write_5m
            .or(self.cache_write_1h)
            .unwrap_or(0.0)
    }

    pub fn cache_read_rate(&self) -> f64 {
        self.cache_read.unwrap_or(0.0)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PriceTable {
    pub version: String,
    pub currency: String,
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub sources: Vec<String>,
    #[serde(default)]
    pub notes: Option<String>,
    pub models: HashMap<String, ModelRates>,
}

impl PriceTable {
    pub fn embedded() -> Self {
        Self::from_json(EMBEDDED_PRICE_TABLE_JSON).expect("embedded price table must parse")
    }

    pub fn from_json(json: &str) -> Result<Self, PriceTableError> {
        Ok(serde_json::from_str(json)?)
    }

    /// Exact resolved lookup (follows `alias_of`). Prefer [`lookup_row`] when the row key is needed.
    pub fn lookup(&self, model: &str) -> Option<&ModelRates> {
        self.lookup_row(model).map(|(rates, _)| rates)
    }

    /// Resolve `alias_of` chain (cycle-safe) → `(canonical rates, row_key)`.
    /// `row_key` is the final non-alias table key used for rates.
    pub fn lookup_row(&self, model: &str) -> Option<(&ModelRates, String)> {
        let mut seen: HashSet<String> = HashSet::new();
        let mut key = model.trim().to_string();
        if key.is_empty() {
            return None;
        }
        loop {
            if !seen.insert(key.clone()) {
                return None; // cycle
            }
            let rates = self.models.get(&key)?;
            match rates.alias_of.clone() {
                Some(alias) if !alias.is_empty() => {
                    key = alias;
                }
                _ => {
                    // Re-borrow rates after owning key for the return row name.
                    let row_key = key.clone();
                    let rates = self.models.get(&row_key)?;
                    return Some((rates, row_key));
                }
            }
        }
    }

    /// Citation string for SpendSummary.price_table_source.
    pub fn source_citation(&self) -> String {
        if self.sources.is_empty() {
            format!("embedded:{}", self.version)
        } else {
            self.sources.join("; ")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alias_resolves_to_target_row_key() {
        let table = PriceTable::embedded();
        let (rates, row) = table.lookup_row("gpt-6-astra").expect("alias");
        assert_eq!(row.as_str(), "gpt-5");
        assert!((rates.input - 1.25).abs() < 1e-9);
    }

    #[test]
    fn unknown_model_returns_none() {
        let table = PriceTable::embedded();
        assert!(table.lookup_row("some-weird-slug").is_none());
    }
}
