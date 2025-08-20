use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, ser::Serializer};
use sqlx::Row;
use uuid::Uuid;
use std::str::FromStr;
use std::fmt;

use crate::types::Decimal;
use bigdecimal::{BigDecimal, FromPrimitive, ToPrimitive};

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub user_id: Uuid,
    pub spbu_id: Uuid,
    pub fuel_type: String,
    pub quantity: Decimal,
    pub price_per_liter: Decimal,
    pub total_price: Decimal,
    pub status: TransactionStatus,
    pub payment_method: String,
    pub payment_status: PaymentStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paid_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, PartialEq, Eq, Clone, Copy)]
#[sqlx(type_name = "transaction_status", rename_all = "lowercase")]
pub enum TransactionStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "paid")]
    Paid,
    #[serde(rename = "processing")]
    Processing,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "cancelled")]
    Cancelled,
}

impl std::str::FromStr for TransactionStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(TransactionStatus::Pending),
            "paid" => Ok(TransactionStatus::Paid),
            "processing" => Ok(TransactionStatus::Processing),
            "completed" => Ok(TransactionStatus::Completed),
            "cancelled" => Ok(TransactionStatus::Cancelled),
            _ => Err(format!("Invalid transaction status: {}", s)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, PartialEq, Eq, Clone, Copy)]
#[sqlx(type_name = "payment_status", rename_all = "lowercase")]
pub enum PaymentStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "paid")]
    Paid,
    #[serde(rename = "failed")]
    Failed,
}

impl std::str::FromStr for PaymentStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(PaymentStatus::Pending),
            "paid" => Ok(PaymentStatus::Paid),
            "failed" => Ok(PaymentStatus::Failed),
            _ => Err(format!("Invalid payment status: {}", s)),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateTransactionRequest {
    pub spbu_id: Uuid,
    pub fuel_type: String,
    #[serde(deserialize_with = "deserialize_decimal")]
    pub quantity: Decimal,  // Use Decimal for precision
    pub payment_method: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub spbu_id: Uuid,
    pub fuel_type: String,
    pub quantity: String,  // Serialized as string for precision
    pub price_per_liter: String,  // Serialized as string for precision
    pub total_price: String,  // Serialized as string for precision
    pub status: String,
    pub payment_method: String,
    pub payment_status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paid_at: Option<DateTime<Utc>>,
}

fn deserialize_decimal<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    
    // Use serde_json's Value to handle both string and number cases without moving the deserializer
    let value = serde_json::Value::deserialize(deserializer)?;
    
    match value {
        serde_json::Value::String(s) => {
            BigDecimal::from_str(&s)
                .map(Decimal)
                .map_err(Error::custom)
        },
        serde_json::Value::Number(num) => {
            if let Some(f) = num.as_f64() {
                Ok(Decimal(BigDecimal::from_f64(f).unwrap_or_default()))
            } else if let Some(i) = num.as_i64() {
                Ok(Decimal(BigDecimal::from(i)))
            } else {
                Err(Error::custom("Invalid number format for decimal"))
            }
        },
        _ => Err(Error::custom("Expected string or number for decimal"))
    }
}

fn decimal_to_f64<S>(value: &Decimal, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use std::str::FromStr;
    
    // Convert BigDecimal to string first to avoid precision loss
    let decimal_str = value.0.to_string();
    let float_val = f64::from_str(&decimal_str).unwrap_or(0.0);
    serializer.serialize_f64(float_val)
}

impl From<Transaction> for TransactionResponse {
    fn from(transaction: Transaction) -> Self {
        TransactionResponse {
            id: transaction.id,
            user_id: transaction.user_id,
            spbu_id: transaction.spbu_id,
            fuel_type: transaction.fuel_type,
            quantity: transaction.quantity.to_string(),
            price_per_liter: transaction.price_per_liter.to_string(),
            total_price: transaction.total_price.to_string(),
            status: transaction.status.to_string(),
            payment_method: transaction.payment_method,
            payment_status: transaction.payment_status.to_string(),
            created_at: transaction.created_at,
            updated_at: transaction.updated_at,
            paid_at: transaction.paid_at,
        }
    }
}

impl fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TransactionStatus::Pending => "pending",
            TransactionStatus::Paid => "paid",
            TransactionStatus::Processing => "processing",
            TransactionStatus::Completed => "completed",
            TransactionStatus::Cancelled => "cancelled",
        };
        write!(f, "{}", s)
    }
}

// Implement FromRow manually for Transaction
impl<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> for Transaction {
    fn from_row(row: &'r sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        // Handle potential NULL values with fallbacks
        let id: Uuid = row.try_get("id")?;
        let user_id: Uuid = row.try_get("user_id")?;
        let spbu_id: Uuid = row.try_get("spbu_id")?;
        let fuel_type: String = row.try_get("fuel_type")?;
        
        // Handle decimal fields with fallback to string parsing if needed
        let quantity = match row.try_get::<String, _>("quantity") {
            Ok(s) => Decimal::from_str(&s).map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
            Err(_) => Decimal::from(0),
        };
        
        let price_per_liter = match row.try_get::<String, _>("price_per_liter") {
            Ok(s) => Decimal::from_str(&s).map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
            Err(_) => Decimal::from(0),
        };
        
        let total_price = match row.try_get::<String, _>("total_price") {
            Ok(s) => Decimal::from_str(&s).map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
            Err(_) => Decimal::from(0),
        };
        
        let status: TransactionStatus = row.try_get("status")?;
        let payment_method: String = row.try_get("payment_method")?;
        let payment_status: PaymentStatus = row.try_get("payment_status")?;
        let created_at: Option<DateTime<Utc>> = row.try_get("created_at").ok();
        let updated_at: Option<DateTime<Utc>> = row.try_get("updated_at").ok();
        let paid_at: Option<DateTime<Utc>> = row.try_get("paid_at").ok();

        Ok(Transaction {
            id,
            user_id,
            spbu_id,
            fuel_type,
            quantity,
            price_per_liter,
            total_price,
            status,
            payment_method,
            payment_status,
            created_at,
            updated_at,
            paid_at,
        })
    }
}

// Serialization helper for Decimal fields
pub fn serialize_decimal<S>(decimal: &Decimal, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&decimal.to_string())
}

impl std::fmt::Display for PaymentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Pending => "pending",
                Self::Paid => "paid",
                Self::Failed => "failed",
            }
        )
    }
}
