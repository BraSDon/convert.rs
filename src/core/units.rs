use std::collections::HashMap;
use std::fmt::Display;
use std::str::FromStr;
use std::sync::Mutex;
use std::{default, mem};

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use super::currency::ConversionCache;
use once_cell::sync::Lazy;

static CACHE: Lazy<Mutex<ConversionCache>> = Lazy::new(|| Mutex::new(ConversionCache::new()));

#[derive(Debug, PartialEq)]
pub struct ConversionError {
    message: String,
}

impl Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Conversion error: {}", self.message)
    }
}

type ConversionResult<T> = Result<T, ConversionError>;

#[derive(Debug, PartialEq)]
pub struct Value {
    value: Option<f64>,
    unit: Unit,
}

impl Value {
    pub fn new(value: f64, unit: Unit) -> Value {
        Value {
            value: Some(value),
            unit,
        }
    }

    pub fn convert_to(&self, to: &Unit) -> ConversionResult<Value> {
        self.value.ok_or(ConversionError {
            message: "Value is None".to_string(),
        })?;
        if self.unit != *to {
            return Err(ConversionError {
                message: format!("Cannot convert from {} to {}", self.unit, to),
            });
        }

        let new_value = Unit::convert(self.value.unwrap(), &self.unit, to)?;
        Ok(Value {
            value: Some(new_value),
            unit: (*to).clone(),
        })
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.value {
            Some(v) => write!(f, "{} {}", v, self.unit),
            None => write!(f, "None {}", self.unit),
        }
    }
}

#[derive(Debug, Clone, Copy, EnumIter)]
pub enum Unit {
    Length(LengthUnit),
    Mass(MassUnit),
    Currency(CurrencyUnit),
}

impl Unit {
    fn convert(value: f64, from: &Unit, to: &Unit) -> ConversionResult<f64> {
        match (from, to) {
            (Unit::Length(from), Unit::Length(to)) => LengthUnit::convert(value, from, to),
            (Unit::Mass(from), Unit::Mass(to)) => MassUnit::convert(value, from, to),
            (Unit::Currency(from), Unit::Currency(to)) => CurrencyUnit::convert(value, from, to),
            _ => Err(ConversionError {
                message: format!("Cannot convert from {} to {}", from, to),
            }),
        }
    }

    pub fn get_all_units() -> Vec<Unit> {
        Unit::iter()
            .flat_map(|unit| match unit {
                Unit::Length(_) => LengthUnit::iter().map(Unit::Length).collect::<Vec<Unit>>(),
                Unit::Mass(_) => MassUnit::iter().map(Unit::Mass).collect::<Vec<Unit>>(),
                Unit::Currency(_) => CurrencyUnit::iter()
                    .map(Unit::Currency)
                    .collect::<Vec<Unit>>(),
            })
            .collect()
    }
}

impl PartialEq for Unit {
    fn eq(&self, other: &Self) -> bool {
        mem::discriminant(self) == mem::discriminant(other)
    }
}

impl Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Unit::Length(u) => write!(f, "{}", u),
            Unit::Mass(u) => write!(f, "{}", u),
            Unit::Currency(u) => write!(f, "{}", u),
        }
    }
}

impl FromStr for Unit {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(length_unit) = s.parse::<LengthUnit>() {
            return Ok(Unit::Length(length_unit));
        }
        if let Ok(mass_unit) = s.parse::<MassUnit>() {
            return Ok(Unit::Mass(mass_unit));
        }
        if let Ok(currency_unit) = s.parse::<CurrencyUnit>() {
            return Ok(Unit::Currency(currency_unit));
        }
        Err(format!("Invalid unit: {}", s))
    }
}

trait Convertable {
    fn to_base_unit(&self, value: f64) -> ConversionResult<f64>;
    fn from_base_unit(&self, value: f64) -> ConversionResult<f64> {
        let base_value = self.to_base_unit(1.0)?;
        Ok(value / base_value)
    }
    fn convert(value: f64, from: &Self, to: &Self) -> ConversionResult<f64> {
        let base_value = from.to_base_unit(value)?;
        to.from_base_unit(base_value)
    }
}

trait Unitlike:
    Display + PartialEq + Convertable + FromStr + default::Default + IntoEnumIterator + Clone + Copy
{
    fn get_display_map() -> HashMap<(&'static str, &'static str), Self>;
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let display_map = Self::get_display_map();
        let (long, short) = display_map.iter().find(|(_, &v)| v == *self).unwrap().0;
        write!(f, "{} ({})", long, short)
    }

    fn from_str(s: &str) -> Result<Self, String> {
        Self::get_display_map()
            .iter()
            .find(|&((long, short), _)| s == *long || s == *short)
            .map(|(_, &unit)| unit)
            .ok_or_else(|| format!("Invalid unit: {}", s))
    }
}

#[derive(Debug, PartialEq, Clone, Copy, EnumIter, Default)]
pub enum LengthUnit {
    #[default]
    Meter,
    Centimeter,
    Kilometer,
    Yard,
    Foot,
    Inch,
}

impl Unitlike for LengthUnit {
    fn get_display_map() -> HashMap<(&'static str, &'static str), LengthUnit> {
        let mut m = HashMap::new();
        m.insert(("meter", "m"), LengthUnit::Meter);
        m.insert(("centimeter", "cm"), LengthUnit::Centimeter);
        m.insert(("kilometer", "km"), LengthUnit::Kilometer);
        m.insert(("yard", "yd"), LengthUnit::Yard);
        m.insert(("foot", "ft"), LengthUnit::Foot);
        m.insert(("inch", "in"), LengthUnit::Inch);
        m
    }
}

impl Display for LengthUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Unitlike::fmt(self, f)
    }
}

impl FromStr for LengthUnit {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Unitlike::from_str(s)
    }
}

impl Convertable for LengthUnit {
    fn to_base_unit(&self, value: f64) -> ConversionResult<f64> {
        let val = match self {
            LengthUnit::Meter => value,
            LengthUnit::Centimeter => value / 100.0,
            LengthUnit::Kilometer => value * 1000.0,
            LengthUnit::Yard => value * 0.9144,
            LengthUnit::Foot => value * 0.3048,
            LengthUnit::Inch => value * 0.0254,
        };
        Ok(val)
    }
}

#[derive(Debug, PartialEq, Clone, Copy, EnumIter, Default)]
pub enum MassUnit {
    #[default]
    Kilogram,
    Gram,
    Ton,
    Pound,
    Ounce,
}

impl Unitlike for MassUnit {
    fn get_display_map() -> HashMap<(&'static str, &'static str), MassUnit> {
        let mut m = HashMap::new();
        m.insert(("kilogram", "kg"), MassUnit::Kilogram);
        m.insert(("gram", "g"), MassUnit::Gram);
        m.insert(("ton", "t"), MassUnit::Ton);
        m.insert(("pound", "lb"), MassUnit::Pound);
        m.insert(("ounce", "oz"), MassUnit::Ounce);
        m
    }
}

impl Display for MassUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Unitlike::fmt(self, f)
    }
}

impl FromStr for MassUnit {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Unitlike::from_str(s)
    }
}

impl Convertable for MassUnit {
    fn to_base_unit(&self, value: f64) -> ConversionResult<f64> {
        let val = match self {
            MassUnit::Kilogram => value,
            MassUnit::Gram => value / 1000.0,
            MassUnit::Ton => value * 1000.0,
            MassUnit::Pound => value * 0.453592,
            MassUnit::Ounce => value * 0.0283495,
        };
        Ok(val)
    }
}

#[derive(Debug, PartialEq, Clone, Copy, EnumIter, Default, Hash, Eq)]
pub enum CurrencyUnit {
    #[default]
    USD,
    EUR,
    JPY,
    KRW,
    GBP,
    AUD,
}

impl Unitlike for CurrencyUnit {
    fn get_display_map() -> HashMap<(&'static str, &'static str), CurrencyUnit> {
        let mut m = HashMap::new();
        m.insert(("USD", "USD"), CurrencyUnit::USD);
        m.insert(("EUR", "EUR"), CurrencyUnit::EUR);
        m.insert(("JPY", "JPY"), CurrencyUnit::JPY);
        m.insert(("KRW", "KRW"), CurrencyUnit::KRW);
        m.insert(("GBP", "GBP"), CurrencyUnit::GBP);
        m.insert(("AUD", "AUD"), CurrencyUnit::AUD);
        m
    }
}

impl Display for CurrencyUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let display_map = Self::get_display_map();
        let (long, _) = display_map.iter().find(|(_, &v)| v == *self).unwrap().0;
        write!(f, "{}", long)
    }
}

impl FromStr for CurrencyUnit {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Unitlike::from_str(s)
    }
}

impl Convertable for CurrencyUnit {
    fn to_base_unit(&self, value: f64) -> ConversionResult<f64> {
        CACHE
            .lock()
            .unwrap()
            .get_base_rate(*self)
            .map(|rate| value / rate)
            .map_err(|e| ConversionError {
                message: e.to_string(),
            })
    }
}

// test eq of value
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_eq() {
        let v1 = Value {
            value: Some(1.0),
            unit: Unit::Length(LengthUnit::Meter),
        };
        let v2 = Value {
            value: Some(1.0),
            unit: Unit::Length(LengthUnit::Meter),
        };

        let v3 = Value {
            value: Some(1.0),
            unit: Unit::Length(LengthUnit::Kilometer),
        };

        let v4 = Value {
            value: Some(2.0),
            unit: Unit::Length(LengthUnit::Meter),
        };

        assert_eq!(v1, v2);
        assert_eq!(v1, v3);
        assert_ne!(v1, v4);
    }

    #[test]
    fn test_unit_eq() {
        let u1 = Unit::Length(LengthUnit::Meter);
        let u2 = Unit::Length(LengthUnit::Meter);
        assert_eq!(u1, u2);

        let u3 = Unit::Length(LengthUnit::Kilometer);
        assert_eq!(u1, u3);

        let u4 = Unit::Mass(MassUnit::Kilogram);
        assert_ne!(u1, u4);
    }

    #[test]
    fn test_length_conversion() {
        let v = Value::new(1.0, Unit::Length(LengthUnit::Meter));
        let v2 = v.convert_to(&Unit::Length(LengthUnit::Kilometer)).unwrap();
        assert_eq!(v2, Value::new(0.001, Unit::Length(LengthUnit::Kilometer)));
    }

    #[test]
    fn test_length_conversion_edge_case() {
        let v = Value::new(0.0, Unit::Length(LengthUnit::Meter));
        let v2 = v.convert_to(&Unit::Length(LengthUnit::Kilometer)).unwrap();
        assert_eq!(v2, Value::new(0.0, Unit::Length(LengthUnit::Kilometer)));
    }

    #[test]
    fn test_mass_conversion() {
        let v = Value::new(1.0, Unit::Mass(MassUnit::Kilogram));
        let v2 = v.convert_to(&Unit::Mass(MassUnit::Gram)).unwrap();
        assert_eq!(v2, Value::new(1000.0, Unit::Mass(MassUnit::Gram)));
    }

    #[test]
    fn test_currency_conversion() {
        let v = Value::new(1.0, Unit::Currency(CurrencyUnit::USD));
        let v2 = v.convert_to(&Unit::Currency(CurrencyUnit::EUR));
        assert!(v2.is_ok());
    }
}
