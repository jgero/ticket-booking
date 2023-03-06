use super::errors::RepoError;
use crate::model::order::Order;
use futures::TryFutureExt;
use log::info;
use sqlx::{Pool, Postgres};
use std::ops::RangeBounds;

// try to realize an order.
// return true if realizing succeeded, false if tickets are already taken
pub async fn realize_order(order: Order, pool: Pool<Postgres>) -> Result<bool, RepoError> {
    let mut transaction = pool
        .begin()
        .await
        .map_err(|e| RepoError::DatabaseError(e.to_string()))?;
    let query_string = format!(
        "SELECT * FROM tickets WHERE visible_id IN ({}) AND ordered_by IS NULL",
        build_query_param_list(1..=order.tickets.len())
    );
    info!("get tickets query: {}", query_string);
    let mut query = sqlx::query(&query_string);
    for ticket in &order.tickets {
        query = query.bind(&ticket.visible_id);
    }
    let available_ticket_rows = query
        .fetch_all(&mut transaction)
        .map_err(|e| RepoError::DatabaseError(e.to_string()))
        .await?;
    if available_ticket_rows.len() != order.tickets.len() {
        return Ok(false);
    }
    let mut query = sqlx::query("INSERT INTO orders VALUES ($1, $2)");
    query = query.bind(order.uuid.to_string());
    query = query.bind(order.issuer);
    let inserted_order_count = query
        .execute(&mut transaction)
        .map_err(|e| RepoError::DatabaseError(e.to_string()))
        .await?;
    if inserted_order_count.rows_affected() != 1 {
        transaction
            .rollback()
            .map_err(|e| RepoError::DatabaseError(e.to_string()))
            .await?;
        return Err(RepoError::DataError(format!(
            "more than one row was modified when inserting order {}",
            order.uuid.to_string()
        )));
    }
    let query_string = format!(
        "UPDATE tickets SET ordered_by=$1 WHERE visible_id IN ({})",
        build_query_param_list(2..2 + order.tickets.len())
    );
    info!("update tickets query: {}", query_string);
    let mut query = sqlx::query(&query_string);
    query = query.bind(order.uuid.to_string());
    for ticket in &order.tickets {
        query = query.bind(&ticket.visible_id);
    }
    let updated_tickets = query
        .execute(&mut transaction)
        .map_err(|e| RepoError::DatabaseError(e.to_string()))
        .await?;
    // or use "as"
    if updated_tickets.rows_affected() != order.tickets.len() as u64 {
        transaction
            .rollback()
            .map_err(|e| RepoError::DatabaseError(e.to_string()))
            .await?;
        return Err(RepoError::DataError(format!(
            "{} rows were updated instead of expected {} when setting order status for order {}",
            updated_tickets.rows_affected(),
            order.tickets.len(),
            order.uuid.to_string()
        )));
    }
    transaction.commit()
            .map_err(|e| RepoError::DatabaseError(e.to_string()))
            .await?;
    Ok(true)
}
fn build_query_param_list<T>(param_range: T) -> String
where
    T: RangeBounds<usize> + Iterator<Item = usize>,
{
    param_range
        .into_iter()
        .map(|i| format!("${}", i))
        .collect::<Vec<String>>()
        .join(",")
}
