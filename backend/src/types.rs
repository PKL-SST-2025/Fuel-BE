use bigdecimal::{BigDecimal, FromPrimitive, ToPrimitive, Zero, num_traits::Num};
use serde::{Serialize, Serializer, Deserialize, Deserializer, de};
use std::ops::{Add, Sub, Mul, Div};
use std::str::FromStr;
use std::fmt;

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct Decimal(pub BigDecimal);

impl Serialize for Decimal {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for Decimal {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Create a visitor that can handle both string and number deserialization
        struct DecimalVisitor;

        impl<'de> de::Visitor<'de> for DecimalVisitor {
            type Value = Decimal;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string or number representing a decimal value")
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                value.parse().map_err(de::Error::custom)
            }

            fn visit_string<E: de::Error>(self, value: String) -> Result<Self::Value, E> {
                self.visit_str(&value)
            }

            fn visit_f64<E: de::Error>(self, value: f64) -> Result<Self::Value, E> {
                BigDecimal::from_f64(value)
                    .map(Decimal)
                    .ok_or_else(|| de::Error::custom("Failed to convert f64 to BigDecimal"))
            }

            fn visit_i64<E: de::Error>(self, value: i64) -> Result<Self::Value, E> {
                Ok(Decimal(BigDecimal::from(value)))
            }

            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
                Ok(Decimal(BigDecimal::from(value)))
            }
        }

        deserializer.deserialize_any(DecimalVisitor)
    }
}

// Implement From traits
impl From<f64> for Decimal {
    fn from(value: f64) -> Self {
        Decimal(BigDecimal::from_f64(value).unwrap_or_else(|| BigDecimal::from(0)))
    }
}

impl From<u64> for Decimal {
    fn from(value: u64) -> Self {
        Decimal(BigDecimal::from(value))
    }
}

impl From<BigDecimal> for Decimal {
    fn from(value: BigDecimal) -> Self {
        Decimal(value)
    }
}

impl From<Decimal> for BigDecimal {
    fn from(val: Decimal) -> Self {
        val.0
    }
}

impl From<i32> for Decimal {
    fn from(value: i32) -> Self {
        Decimal(BigDecimal::from(value))
    }
}

// Implement arithmetic operations
impl Add for Decimal {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Decimal(self.0 + other.0)
    }
}

impl Sub for Decimal {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Decimal(self.0 - other.0)
    }
}

impl Mul for Decimal {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Decimal(self.0 * other.0)
    }
}

impl Div for Decimal {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        if other.0.is_zero() {
            panic!("Division by zero");
        }
        Decimal(self.0 / other.0)
    }
}

// Implement FromStr
impl FromStr for Decimal {
    type Err = bigdecimal::ParseBigDecimalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        BigDecimal::from_str(s).map(Decimal)
    }
}

// Implement Display
impl std::fmt::Display for Decimal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Implement SQLx Type for PostgreSQL
#[cfg(feature = "sqlx-postgres")]
impl sqlx::Type<sqlx::Postgres> for Decimal {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("NUMERIC")
    }
}

#[cfg(feature = "sqlx-postgres")]
impl<'r> sqlx::Decode<'r, sqlx::Postgres> for Decimal {
    fn decode(value: <sqlx::Postgres as sqlx::database::HasValueRef<'r>>::ValueRef) -> Result<Self, sqlx::error::BoxDynError> {
        let value = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        let decimal = BigDecimal::from_str(value)?;
        Ok(Decimal(decimal))
    }
}

#[cfg(feature = "sqlx-postgres")]
impl<'q> sqlx::Encode<'q, sqlx::Postgres> for Decimal {
    fn encode_by_ref(&self, buf: &mut <sqlx::Postgres as sqlx::database::HasArguments<'q>>::ArgumentBuffer) -> sqlx::encode::IsNull {
        let s = self.0.to_string();
        <&str as sqlx::Encode<sqlx::Postgres>>::encode(s.as_str(), buf)
    }
}
