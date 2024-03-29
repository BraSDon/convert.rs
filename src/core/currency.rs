/*
TODO:
- Persist conversion rates using a lightweight database
*/

use super::units::CurrencyUnit;
use chrono::{DateTime, TimeDelta, Utc};
use reqwest;
use serde_json::Value;
use std::{collections::HashMap, fmt::Display};

const API_BASE_URL: &str = "https://openexchangerates.org/api/latest.json";

pub struct ConversionCache {
    /// Map from starting currency to base currency (USD) and timestamp of last update
    cache: HashMap<CurrencyUnit, f64>,
    /// Time after which a cache line expires
    expire_after: TimeDelta,
    last_time: Option<DateTime<Utc>>,
}

impl ConversionCache {
    /// Create a new ConversionCache with a given expiration time.
    pub fn new() -> Self {
        ConversionCache {
            cache: HashMap::new(),
            expire_after: TimeDelta::new(3600, 0).unwrap(),
            last_time: None,
        }
    }

    /// Get the conversion rate from USD to a given currency.
    /// I.e. how many fromUnit is one USD worth?
    pub fn get_base_rate(&mut self, from: CurrencyUnit) -> Result<f64, APIError> {
        if self.last_time.is_none() || self.last_time.unwrap() + self.expire_after < Utc::now() {
            self.request_and_update(from)
        } else {
            let entry = self.cache.get(&from);
            match entry {
                Some(rate) => Ok(*rate),
                None => self.request_and_update(from),
            }
        }
    }

    /// Request the conversion rate from the API and update the cache accordingly.
    fn request_and_update(&mut self, from: CurrencyUnit) -> Result<f64, APIError> {
        let response = self.request()?;
        self.update(response)?;
        self.cache.get(&from).cloned().ok_or(APIError {
            message: "Rate not found".to_string(),
        })
    }

    fn request(&self) -> Result<Value, APIError> {
        let app_id = std::env::var("OPENEXCHANGERATES_APP_ID").map_err(|_| APIError {
            message: "API key not found".to_string(),
        })?;
        let body = reqwest::blocking::get(&format!("{}?app_id={}", API_BASE_URL, app_id))?
            .json::<serde_json::Value>()?;
        Ok(body)
    }

    fn update(&mut self, response: Value) -> Result<(), APIError> {
        let timestamp: DateTime<Utc> = response["timestamp"]
            .as_i64()
            .map(|n| DateTime::from_timestamp(n, 0))
            .unwrap_or_else(|| Some(Utc::now()))
            .unwrap(); // Never panics because Utc::now() always works

        let rates = response["rates"].as_object().ok_or(APIError {
            message: "Rates not found".to_string(),
        })?;

        for (currency, rate) in rates {
            let rate = rate.as_f64().ok_or(APIError {
                message: "Invalid rate format".to_string(),
            })?;
            match currency.parse() {
                Ok(currency) => {
                    self.cache.insert(currency, rate);
                    Some(())
                }
                Err(_) => continue,
            };
        }
        self.last_time = Some(timestamp);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct APIError {
    message: String,
}

impl Display for APIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "API error: {}", self.message)
    }
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
        let mut cache = ConversionCache::new();
        let rate = cache.get_base_rate(CurrencyUnit::EUR);
        assert!(rate.is_ok());

        let rate_new = cache.get_base_rate(CurrencyUnit::EUR);
        assert!(rate_new.is_ok());
        assert!(rate.unwrap() == rate_new.unwrap());
        // TODO: assert that request_and_update was NOT called
    }
}
