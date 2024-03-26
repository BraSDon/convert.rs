/*
TODO:
- Persist conversion rates using a lightweight database
 */

use std::collections::HashMap;
use chrono::{DateTime, Utc, TimeDelta};
use reqwest;
use serde_json::Number;
use super::units::CurrencyUnit;

const API_BASE_URL: &str = "https://openexchangerates.org/api/latest.json";

pub struct ConversionCache {
    /// Map from starting currency to base currency (USD) and timestamp of last update
    cache: HashMap<CurrencyUnit, (f64, DateTime<Utc>)>,
    /// Time after which a cache line expires
    expire_after: TimeDelta,
}

impl ConversionCache {
    /// Create a new ConversionCache with a given expiration time.
    pub fn new(expire_after: TimeDelta) -> Self {
        ConversionCache {
            cache: HashMap::new(),
            expire_after,
        }
    }

    /// Get the conversion rate from one currency to USD.
    pub fn get_base_rate(&mut self, from: CurrencyUnit) -> Result<f64, APIError> {
        self.cache.get(&from)
            .filter(|(_, timestamp)| *timestamp + self.expire_after > Utc::now())
            .map(|(rate, _)| Ok(*rate))
            .unwrap_or_else(|| self.request_and_update(from))
    }

    /// Request the conversion rate from the API and update the cache accordingly.
    fn request_and_update(&mut self, from: CurrencyUnit) -> Result<f64, APIError> {
        let app_id = std::env::var("OPENEXCHANGERATES_APP_ID").map_err(|_| APIError {
            message: "No API key found. Please set the OPENEXCHANGERATES_APP_ID environment variable.".to_string(),
        })?;
        let body = reqwest::blocking::get(&format!("{}?app_id={}", API_BASE_URL, app_id))?
            .json::<serde_json::Value>()?;

        let rate = body["rates"][from.to_string()].as_f64().ok_or(APIError {
            message: format!("No rate found for currency {}", from),
        })?;
        let timestamp: DateTime<Utc> = body["timestamp"].as_i64()
            .map(|n| DateTime::from_timestamp(n, 0))
            .unwrap_or_else(|| Some(Utc::now()))
            .unwrap(); // Never panics because Utc::now() always works

        self.cache.insert(from, (rate, timestamp));
        Ok(rate)
    }
}

#[derive(Debug, Clone)]
pub struct APIError {
    message: String,
}

impl From<reqwest::Error> for APIError {
    fn from(e: reqwest::Error) -> Self {
        APIError {
            message: e.to_string(),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_entry_multiple_times() {
        let mut cache = ConversionCache::new(TimeDelta::new(100, 0).unwrap());
        let rate = cache.get_base_rate(CurrencyUnit::EUR);
        assert!(rate.is_ok());

        let rate_new = cache.get_base_rate(CurrencyUnit::EUR);
        assert!(rate_new.is_ok());
        assert!(rate.unwrap() == rate_new.unwrap());
        // TODO: assert that request_and_update was NOT called
    }
}