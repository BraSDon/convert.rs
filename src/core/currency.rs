use super::units::CurrencyUnit;
use chrono::{DateTime, TimeDelta, Utc};
use reqwest;
use rusqlite::{Connection, Result};
use serde_json::Value;
use std::{collections::HashMap, fmt::Display};

const API_BASE_URL: &str = "https://openexchangerates.org/api/latest.json";
const EXPIRE_AFTER: i64 = 60 * 60 * 24 * 7; // 1 week

pub struct ConversionCache {
    /// Map from starting currency to base currency (USD) and timestamp of last update
    cache: HashMap<CurrencyUnit, f64>,
    /// Time after which a cache line expires
    expire_after: TimeDelta,
    last_time: Option<DateTime<Utc>>,
}

impl Default for ConversionCache {
    fn default() -> Self {
        ConversionCache {
            cache: HashMap::new(),
            expire_after: TimeDelta::new(EXPIRE_AFTER, 0).unwrap(),
            last_time: None,
        }
    }
}

impl ConversionCache {
    /// Create a new ConversionCache with a given expiration time.
    pub fn new() -> Self {
        match Self::load_from_db() {
            Ok(cache) => cache,
            Err(_) => Self::default(),
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

    /// Request conversion rates from USD to all other currencies.
    fn request(&self) -> Result<Value, APIError> {
        let app_id = std::env::var("OPENEXCHANGERATES_APP_ID").map_err(|_| APIError {
            message: "API key not found".to_string(),
        })?;
        let body = reqwest::blocking::get(format!("{}?app_id={}", API_BASE_URL, app_id))?
            .json::<serde_json::Value>()?;
        Ok(body)
    }

    /// Update the cache with the given response.
    /// The response should be the JSON object returned by the specified API.
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
        let _ = self.save_to_db();
        Ok(())
    }

    /// Save the cache to the database.
    fn save_to_db(&self) -> Result<()> {
        let conn = Connection::open("conversion_cache.db")?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS conversion_cache (
                currency TEXT PRIMARY KEY,
                rate REAL,
                last_update TEXT
            )",
            [],
        )?;

        for (currency, rate) in self.cache.iter() {
            conn.execute(
                "INSERT OR REPLACE INTO conversion_cache (currency, rate, last_update)
                VALUES (?, ?, ?)",
                [
                    currency.to_string(),
                    rate.to_string(),
                    self.last_time.unwrap().to_string(),
                ],
            )?;
        }
        Ok(())
    }

    /// Load the cache from the database.
    fn load_from_db() -> Result<Self, Box<dyn std::error::Error>> {
        let conn = Connection::open("conversion_cache.db")?;
        let mut stmt = conn.prepare("SELECT * FROM conversion_cache")?;
        let rows = stmt.query_map([], |row| {
            let currency: String = row.get(0)?;
            let rate: f64 = row.get(1)?;
            let last_update_str: String = row.get(2)?;

            let currency_unit = currency.parse().expect("Invalid currency unit");
            let last_update = last_update_str.parse().expect("Invalid timestamp");

            Ok((currency_unit, rate, last_update))
        })?;

        let mut cache: HashMap<CurrencyUnit, f64> = HashMap::new();
        let mut last_update = Utc::now(); // Initialize last_update with a default value
        for row_result in rows {
            let (currency, rate, last_update_from_row) = row_result?;
            cache.insert(currency, rate);
            last_update = last_update_from_row;
        }
        Ok(ConversionCache {
            cache,
            expire_after: TimeDelta::new(EXPIRE_AFTER, 0).unwrap(),
            last_time: Some(last_update),
        })
    }
}

#[derive(Debug, Clone)]
pub struct APIError {
    /// Error type for API requests.
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
    use std::time::Instant;

    use super::*;
    use serde_json::json;

    #[test]
    fn test_get_entry_multiple_times() {
        let mut cache = ConversionCache::new();
        let start = Instant::now();
        let rate = cache.get_base_rate(CurrencyUnit::EUR);
        let duration_fst = start.elapsed();
        assert!(rate.is_ok());

        let repeat_count = 10;
        let mut total_duration = std::time::Duration::new(0, 0);
        for _ in 0..repeat_count {
            let start = Instant::now();
            let rate_new = cache.get_base_rate(CurrencyUnit::EUR);
            total_duration += start.elapsed();
            assert!(rate_new.is_ok());
            assert!(rate.clone().unwrap() == rate_new.unwrap());
        }

        let average_duration = total_duration / repeat_count;

        // implicitly check that subsequent calls do not require a new API request,
        // therefore should be faster than the first call.
        assert!(duration_fst > average_duration);
    }

    #[test]
    fn test_update_with_valid_response() {
        let mut cache = ConversionCache::new();
        let response = json!({
            "timestamp": Utc::now().timestamp(),
            "rates": {
                "EUR": 1.0,
                "USD": 1.2
            }
        });
        assert!(cache.update(response).is_ok());
    }

    #[test]
    fn test_update_with_invalid_rate() {
        let mut cache = ConversionCache::new();
        let response = json!({
            "timestamp": Utc::now().timestamp(),
            "rates": {
                "EUR": "invalid",
                "USD": 1.2
            }
        });
        assert!(cache.update(response).is_err());
    }

    #[test]
    fn test_update_with_invalid_timestamp() {
        let mut cache = ConversionCache::new();
        let response = json!({
            "timestamp": "invalid",
            "rates": {
                "EUR": 1.0,
                "USD": 1.2
            }
        });
        assert!(cache.update(response).is_ok());
    }

    #[test]
    fn test_save_to_db_and_load_from_db() {
        let mut cache = ConversionCache::new();
        let response = json!({
            "timestamp": Utc::now().timestamp(),
            "rates": {
                "EUR": 1.0,
                "USD": 1.2
            }
        });
        assert!(cache.update(response).is_ok());
        assert!(cache.save_to_db().is_ok());

        let loaded_cache = ConversionCache::load_from_db();
        assert!(loaded_cache.is_ok());
        assert_eq!(cache.cache, loaded_cache.unwrap().cache);
    }

    #[test]
    fn test_api_error_display() {
        let error = APIError {
            message: "Test error".to_string(),
        };
        assert_eq!(format!("{}", error), "API error: Test error");
    }
}
