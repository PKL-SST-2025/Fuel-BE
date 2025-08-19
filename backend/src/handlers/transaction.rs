use axum::{
    extract::{Path, State, Extension},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum::extract::FromRef;
use bigdecimal::{BigDecimal, FromPrimitive};
use serde_json::json;
use std::str::FromStr;
use uuid::Uuid;

use crate::types::Decimal;
use crate::{
    models::{CreateTransactionRequest, Transaction, TransactionResponse, TransactionStatus, PaymentStatus},
    AppState,
};
use sqlx::Row;

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    NotFound(String),
    InternalServerError(String),
    DatabaseError(String),
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::DatabaseError(err.to_string())
    }
}

impl From<(StatusCode, String)> for AppError {
    fn from((status, message): (StatusCode, String)) -> Self {
        match status {
            StatusCode::BAD_REQUEST => AppError::BadRequest(message),
            StatusCode::NOT_FOUND => AppError::NotFound(message),
            _ => AppError::InternalServerError(message),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            AppError::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
            AppError::NotFound(message) => (StatusCode::NOT_FOUND, message),
            AppError::InternalServerError(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
            AppError::DatabaseError(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
            AppError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            AppError::InternalServerError(msg) => write!(f, "Internal Server Error: {}", msg),
            AppError::DatabaseError(msg) => write!(f, "Database Error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

type Result<T> = std::result::Result<T, AppError>;

#[axum::debug_handler]
pub async fn create_transaction(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Json(payload): Json<CreateTransactionRequest>,
) -> Result<Json<TransactionResponse>> {
    // Log the user_id for debugging
    println!("Attempting to create transaction for user_id: {}", user_id);
    // Check if user exists
    println!("Checking if user exists in database...");
    let user_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)"
    )
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        println!("Database error when checking user: {}", e);
        e
    })?;

    if !user_exists {
        println!("User not found in database: {}", user_id);
        return Err(AppError::BadRequest("User not found".to_string()));
    }
    
    println!("User found in database");

    // Validate quantity
    if payload.quantity.0 <= BigDecimal::from(0) {
        return Err(AppError::BadRequest("Quantity must be greater than 0".to_string()));
    }

    // Check if SPBU exists
    let spbu_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM spbu WHERE id = $1)"
    )
    .bind(payload.spbu_id)
    .fetch_one(&state.db)
    .await?;

    if !spbu_exists {
        return Err(AppError::BadRequest("SPBU not found".to_string()));
    }

    // Get price per liter from the database based on SPBU and fuel type
    let fuel_price = sqlx::query_scalar!(
        r#"
        SELECT price FROM fuel_prices 
        WHERE spbu_id = $1 AND fuel_type = $2
        "#,
        payload.spbu_id,
        &payload.fuel_type
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::BadRequest("Fuel price not found for the specified SPBU and fuel type".to_string()))?;
    
    // Parse the price from the database
    let price_per_liter = Decimal::from_str(&fuel_price.to_string())
        .map_err(|_| AppError::InternalServerError("Invalid price format in database".to_string()))?;
    
    // Clone the BigDecimal values before using them in multiplication
    let price_per_liter_bd = price_per_liter.0.clone();
    let quantity_bd = payload.quantity.0.clone();
    let total_price = Decimal(price_per_liter_bd * quantity_bd);
    
    // Convert to string for database storage
    let quantity_str = payload.quantity.0.to_string();
    let price_per_liter_str = price_per_liter.0.to_string();
    let total_price_str = total_price.0.to_string();

    // Insert transaction and get the ID
    let transaction_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO transactions (
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
            updated_at
        ) VALUES ($1, $2, $3, $4::numeric, $5::numeric, $6::numeric, $7::transaction_status, $8, $9::payment_status, NOW(), NOW())
        RETURNING id
        "#
    )
    .bind(user_id)
    .bind(payload.spbu_id)
    .bind(&payload.fuel_type)
    .bind(quantity_str)
    .bind(price_per_liter_str)
    .bind(total_price_str)
    .bind(TransactionStatus::Pending)
    .bind(&payload.payment_method)
    .bind(PaymentStatus::Pending)
    .fetch_one(&state.db)
    .await?;
    
    // Get the full transaction details
    let transaction = sqlx::query_as::<_, Transaction>(
        r#"
        SELECT * FROM transactions WHERE id = $1
        "#
    )
    .bind(transaction_id)
    .fetch_one(&state.db)
    .await?;
    
    Ok(Json(transaction.into()))
}

#[axum::debug_handler]
pub async fn get_transaction(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(transaction_id): Path<Uuid>,
) -> Result<Json<TransactionResponse>> {
    // Get transaction by ID and user ID
    let transaction = sqlx::query(
        r#"
        SELECT 
            id, 
            user_id, 
            spbu_id, 
            fuel_type, 
            quantity::text, 
            price_per_liter::text, 
            total_price::text, 
            status, 
            payment_method, 
            payment_status, 
            created_at, 
            updated_at, 
            paid_at 
        FROM transactions 
        WHERE id = $1 AND user_id = $2
        "#
    )
    .bind(transaction_id)
    .bind(user_id)
    .map(|row: sqlx::postgres::PgRow| {
        // Manual mapping to handle Decimal fields properly
        Transaction {
            id: row.get("id"),
            user_id: row.get("user_id"),
            spbu_id: row.get("spbu_id"),
            fuel_type: row.get("fuel_type"),
            quantity: Decimal(BigDecimal::from_str(&row.get::<String, _>("quantity")).unwrap_or_default()),
            price_per_liter: Decimal(BigDecimal::from_str(&row.get::<String, _>("price_per_liter")).unwrap_or_default()),
            total_price: Decimal(BigDecimal::from_str(&row.get::<String, _>("total_price")).unwrap_or_default()),
            status: row.get("status"),
            payment_method: row.get("payment_method"),
            payment_status: row.get("payment_status"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            paid_at: row.get("paid_at"),
        }
    })
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    match transaction {
        Some(tx) => Ok(Json(tx.into())),
        None => Err(AppError::NotFound("Transaction not found".to_string())),
    }
}

#[axum::debug_handler]
pub async fn list_transactions(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<Vec<TransactionResponse>>> {
    // Get all transactions for the user, ordered by creation date (newest first)
    let transactions = sqlx::query(
        r#"
        SELECT 
            id, 
            user_id, 
            spbu_id, 
            fuel_type, 
            quantity::text, 
            price_per_liter::text, 
            total_price::text, 
            status, 
            payment_method, 
            payment_status, 
            created_at, 
            updated_at, 
            paid_at 
        FROM transactions 
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#
    )
    .bind(user_id)
    .map(|row: sqlx::postgres::PgRow| {
        // Manual mapping to handle Decimal fields properly
        Transaction {
            id: row.get("id"),
            user_id: row.get("user_id"),
            spbu_id: row.get("spbu_id"),
            fuel_type: row.get("fuel_type"),
            quantity: Decimal::from_str(&row.get::<String, _>("quantity")).unwrap_or_default(),
            price_per_liter: Decimal::from_str(&row.get::<String, _>("price_per_liter")).unwrap_or_default(),
            total_price: Decimal::from_str(&row.get::<String, _>("total_price")).unwrap_or_default(),
            status: row.get("status"),
            payment_method: row.get("payment_method"),
            payment_status: row.get("payment_status"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            paid_at: row.get("paid_at"),
        }
    })
    .fetch_all(&state.db)
    .await?
    .into_iter()
    .map(|tx| tx.into())
    .collect();

    Ok(Json(transactions))
}

#[axum::debug_handler]
pub async fn cancel_transaction(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(transaction_id): Path<Uuid>,
) -> Result<Json<TransactionResponse>> {
    // Start a database transaction to ensure data consistency
    let mut tx = state.db.begin().await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Get the transaction with row lock to prevent race conditions
    let transaction = sqlx::query_as::<_, Transaction>(
        r#"
        SELECT * FROM transactions 
        WHERE id = $1 AND user_id = $2
        FOR UPDATE
        "#
    )
    .bind(transaction_id)
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    match transaction {
        Some(transaction) => {
            // Check if the transaction can be cancelled
            if transaction.status != TransactionStatus::Pending {
                return Err(AppError::BadRequest(
                    "Only pending transactions can be cancelled".to_string(),
                ));
            }

            // Update the transaction status to cancelled
            let updated_transaction = sqlx::query(
                r#"
                UPDATE transactions
                SET status = $1, updated_at = NOW()
                WHERE id = $2 AND user_id = $3
                RETURNING 
                    id, 
                    user_id, 
                    spbu_id, 
                    fuel_type, 
                    quantity::text, 
                    price_per_liter::text, 
                    total_price::text, 
                    status, 
                    payment_method, 
                    payment_status, 
                    created_at, 
                    updated_at, 
                    paid_at
                "#
            )
            .bind(TransactionStatus::Cancelled.to_string())
            .bind(transaction_id)
            .bind(user_id)
            .map(|row: sqlx::postgres::PgRow| {
                // Manual mapping to handle Decimal fields properly
                Transaction {
                    id: row.get("id"),
                    user_id: row.get("user_id"),
                    spbu_id: row.get("spbu_id"),
                    fuel_type: row.get("fuel_type"),
                    quantity: Decimal::from_str(&row.get::<String, _>("quantity")).unwrap_or_default(),
                    price_per_liter: Decimal::from_str(&row.get::<String, _>("price_per_liter")).unwrap_or_default(),
                    total_price: Decimal::from_str(&row.get::<String, _>("total_price")).unwrap_or_default(),
                    status: row.get("status"),
                    payment_method: row.get("payment_method"),
                    payment_status: row.get("payment_status"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                    paid_at: row.get("paid_at"),
                }
            })
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

            // Commit the transaction
            tx.commit().await
                .map_err(|e| AppError::InternalServerError(e.to_string()))?;

            Ok(Json(updated_transaction.into()))
        }
        None => {
            // No transaction found with the given ID and user ID
            Err(AppError::NotFound("Transaction not found".to_string()))
        }
    }
}

#[axum::debug_handler]
pub async fn process_payment(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(transaction_id): Path<Uuid>,
) -> Result<Json<TransactionResponse>> {
    // Start database transaction
    let mut tx = state.db.begin().await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Get the transaction with row lock to prevent race conditions
    let transaction = sqlx::query_as::<_, Transaction>(
        r#"
        SELECT * FROM transactions 
        WHERE id = $1 AND user_id = $2
        FOR UPDATE
        "#
    )
    .bind(transaction_id)
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let transaction = match transaction {
        Some(tx) => tx,
        None => return Err(AppError::NotFound("Transaction not found".to_string())),
    };

    // Validate transaction status
    if transaction.status != TransactionStatus::Pending {
        return Err(AppError::BadRequest(
            "Only pending transactions can be processed".to_string(),
        ));
    }

    // Process payment (in a real implementation, this would call a payment gateway)
    // For demo purposes, we'll assume the payment is always successful
    let payment_success = true;

    let updated_transaction = if payment_success {
        // Update transaction and payment status to paid
        sqlx::query(
            r#"
            UPDATE transactions
            SET status = $1::transaction_status, 
                payment_status = $2::payment_status, 
                paid_at = NOW(),
                updated_at = NOW()
            WHERE id = $3
            RETURNING 
                id, 
                user_id, 
                spbu_id, 
                fuel_type, 
                quantity::text, 
                price_per_liter::text, 
                total_price::text, 
                status, 
                payment_method, 
                payment_status, 
                created_at, 
                updated_at, 
                paid_at
            "#
        )
        .bind(TransactionStatus::Processing)
        .bind(PaymentStatus::Paid)
        .bind(transaction_id)
        .map(|row: sqlx::postgres::PgRow| {
            // Manual mapping to handle Decimal fields properly
            Transaction {
                id: row.get("id"),
                user_id: row.get("user_id"),
                spbu_id: row.get("spbu_id"),
                fuel_type: row.get("fuel_type"),
                quantity: Decimal(BigDecimal::from_str(&row.get::<String, _>("quantity")).unwrap_or_default()),
                price_per_liter: Decimal(BigDecimal::from_str(&row.get::<String, _>("price_per_liter")).unwrap_or_default()),
                total_price: Decimal(BigDecimal::from_str(&row.get::<String, _>("total_price")).unwrap_or_default()),
                status: row.get("status"),
                payment_method: row.get("payment_method"),
                payment_status: row.get("payment_status"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                paid_at: row.get("paid_at"),
            }
        })
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
    } else {
        // If payment failed
        sqlx::query(
            r#"
            UPDATE transactions
            SET payment_status = $1::payment_status, 
                updated_at = NOW()
            WHERE id = $2
            RETURNING 
                id, 
                user_id, 
                spbu_id, 
                fuel_type, 
                quantity::text, 
                price_per_liter::text, 
                total_price::text, 
                status, 
                payment_method, 
                payment_status, 
                created_at, 
                updated_at, 
                paid_at
            "#
        )
        .bind(PaymentStatus::Failed)
        .bind(transaction_id)
        .map(|row: sqlx::postgres::PgRow| {
            // Manual mapping to handle Decimal fields properly
            Transaction {
                id: row.get("id"),
                user_id: row.get("user_id"),
                spbu_id: row.get("spbu_id"),
                fuel_type: row.get("fuel_type"),
                quantity: Decimal(BigDecimal::from_str(&row.get::<String, _>("quantity")).unwrap_or_default()),
                price_per_liter: Decimal(BigDecimal::from_str(&row.get::<String, _>("price_per_liter")).unwrap_or_default()),
                total_price: Decimal(BigDecimal::from_str(&row.get::<String, _>("total_price")).unwrap_or_default()),
                status: row.get("status"),
                payment_method: row.get("payment_method"),
                payment_status: row.get("payment_status"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                paid_at: row.get("paid_at"),
            }
        })
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
    };

    // Commit the transaction
    tx.commit().await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Send notification (in a real implementation, this would send an email/notification)
    if payment_success {
        println!("Payment successful for transaction: {}", transaction_id);
    } else {
        println!("Payment failed for transaction: {}", transaction_id);
    }

    Ok(Json(updated_transaction.into()))
}
