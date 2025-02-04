// Copyright © Aptos Foundation

// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

// This is required because a diesel macro makes clippy sad
#![allow(clippy::extra_unused_lifetimes)]
#![allow(clippy::unused_unit)]
use field_count::FieldCount;
use serde::{Deserialize, Serialize};
use crate::schema::change_table_items_partition;
use diesel::sql_types::{BigInt, Jsonb, Text, Nullable};
use std::env;

#[derive(Clone, Debug, Deserialize, Serialize, Queryable, FieldCount, Insertable)]
#[diesel(table_name = change_table_items_partition)]
pub struct ChangeTableItem {
    pub transaction_version: i64,
    pub change_index: i64,
    pub transaction_timestamp: chrono::NaiveDateTime,
    pub table_handle: String,
    pub key: serde_json::Value,
    pub prev_transaction_version: Option<i64>,
    pub prev_change_index: Option<i64>,
    pub next_transaction_version: Option<i64>,
    pub next_change_index: Option<i64>,
    pub runner_id: i64
}

impl ChangeTableItem {
    pub fn from_transaction(
        transaction_version: i64,
        change_index: i64,
        transaction_timestamp: chrono::NaiveDateTime,
        table_handle: &str,
        key: serde_json::Value,
    ) -> Self {
        let runner_id: i32 = env::var("RUNNER_ID")
        .unwrap_or("0".to_string()) // Default to "0" if the environment variable is not set
        .parse()
        .unwrap_or(0); 
        ChangeTableItem {
            transaction_version,
            change_index,
            transaction_timestamp,
            table_handle: table_handle.to_string(),
            key: key.clone(),
            prev_transaction_version : None,
            prev_change_index : None,
            next_transaction_version : None,
            next_change_index : None,
            runner_id: runner_id as i64
        }
    }
}

#[derive(Debug, QueryableByName, Clone, QueryId, Deserialize, Serialize)]
pub struct ChangeTableItemDiff {
    #[diesel(sql_type = BigInt)]
    pub current_transaction_version: i64,
    #[diesel(sql_type = BigInt)]
    pub current_change_index: i64,
    #[diesel(sql_type = Text)]
    pub table_handle: String,
    #[diesel(sql_type = Jsonb)]
    pub key: serde_json::Value,
    #[diesel(sql_type = Nullable<BigInt>)]
    pub prev_transaction_version: Option<i64>,
    #[diesel(sql_type = Nullable<BigInt>)]
    pub prev_change_index: Option<i64>,
    #[diesel(sql_type = Nullable<BigInt>)]
    pub next_transaction_version: Option<i64>,
    #[diesel(sql_type = Nullable<BigInt>)]
    pub next_change_index: Option<i64>,
}

impl ChangeTableItemDiff {
    /// Creates a new `ChangeTableItemDiff` from transaction details
    pub fn from_transaction(
        current_transaction_version: i64,
        current_change_index: i64,
        table_handle: &str,
        key: serde_json::Value,
        prev_transaction_version: Option<i64>,
        prev_change_index: Option<i64>,
        next_transaction_version: Option<i64>,
        next_change_index: Option<i64>,
    ) -> Self {
        ChangeTableItemDiff {
            current_transaction_version,
            current_change_index,
            table_handle: table_handle.to_string(),
            key: key.clone(),
            prev_transaction_version,
            prev_change_index,
            next_transaction_version,
            next_change_index
        }
    }

    pub fn into_change_table_item(
        self
    ) -> ChangeTableItem {
        let runner_id: i32 = env::var("RUNNER_ID")
        .unwrap_or("0".to_string()) // Default to "0" if the environment variable is not set
        .parse()
        .unwrap_or(0);
        ChangeTableItem { 
                transaction_version: self.current_transaction_version,
                change_index: self.current_change_index,
                #[allow(deprecated)]
                transaction_timestamp: chrono::NaiveDateTime::from_timestamp(0, 0),
                table_handle: self.table_handle,
                key: self.key,
                prev_transaction_version: self.prev_transaction_version,
                prev_change_index: self.prev_change_index,
                next_transaction_version: self.next_transaction_version,
                next_change_index: self.next_change_index,
                runner_id: runner_id as i64
            }
      
    }

}



// Prevent conflicts with other things named `ChangeTableItem`
pub type ChangeTableItemModel = ChangeTableItem;

// Prevent conflicts with other things named `ChangeTableItemDiff`
pub type ChangeTableItemDiffModel = ChangeTableItemDiff;