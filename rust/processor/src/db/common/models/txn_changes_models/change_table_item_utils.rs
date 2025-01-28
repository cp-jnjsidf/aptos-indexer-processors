// Copyright © Aptos Foundation

// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

// This is required because a diesel macro makes clippy sad
#![allow(clippy::extra_unused_lifetimes)]
#![allow(clippy::unused_unit)]
use field_count::FieldCount;
use serde::{Deserialize, Serialize};
use crate::schema::change_table_items_partition;
use diesel::sql_types::{BigInt, Bool, Jsonb, Text, Nullable};
use std::env;

#[derive(Clone, Debug, Deserialize, Serialize, Queryable, FieldCount, Insertable)]
#[diesel(table_name = change_table_items_partition)]
pub struct ChangeTableItem {
    pub transaction_version: i64,
    pub transaction_block_height : i64,
    pub change_index: i64,
    pub transaction_hash: String,
    pub transaction_timestamp: chrono::NaiveDateTime,
    pub transaction_sender: Option<String>,
    pub transaction_entry_function_id_str: Option<String>,
    pub is_transaction_success: bool,
    pub state_key_hash: String,
    pub is_delete: bool,
    pub table_handle: String,
    pub key_type: String,
    pub key: serde_json::Value,
    pub value_type: Option<String>,
    pub value: Option<serde_json::Value>,
    pub prev_transaction_version: Option<i64>,
    pub prev_change_index: Option<i64>,
    pub prev_is_delete: Option<bool>,
    pub prev_value: Option<serde_json::Value>,
    pub next_transaction_version: Option<i64>,
    pub next_change_index: Option<i64>,
    pub next_is_delete: Option<bool>,
    pub next_value: Option<serde_json::Value>,
    pub runner_id: i64
}

impl ChangeTableItem {
    pub fn from_transaction(
        transaction_version: i64,
        transaction_block_height: i64,
        change_index: i64,
        transaction_hash: &str,
        transaction_timestamp: chrono::NaiveDateTime,
        transaction_sender : &Option<String>,
        transaction_entry_function_id_str : &Option<String>,
        is_transaction_success: bool,
        state_key_hash: &str,
        is_delete: bool,
        table_handle: &str,
        key_type: &str,
        key: serde_json::Value,
        value_type: &Option<String>,
        value: &Option<serde_json::Value>
    ) -> Self {
        let runner_id: i32 = env::var("RUNNER_ID")
        .unwrap_or("0".to_string()) // Default to "0" if the environment variable is not set
        .parse()
        .unwrap_or(0); 
        ChangeTableItem {
            transaction_version,
            transaction_block_height,
            change_index,
            transaction_hash: transaction_hash.to_string(),
            transaction_timestamp,
            transaction_sender: transaction_sender.clone(),
            transaction_entry_function_id_str: transaction_entry_function_id_str.clone(),
            is_transaction_success,
            state_key_hash: state_key_hash.to_string(),
            is_delete,
            table_handle: table_handle.to_string(),
            key_type: key_type.to_string(),
            key: key.clone(),
            value_type: value_type.clone(),
            value: value.clone(),
            prev_transaction_version : None,
            prev_change_index : None,
            prev_is_delete : None,
            prev_value : None,
            next_transaction_version : None,
            next_change_index : None,
            next_is_delete : None,
            next_value : None,
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
    #[diesel(sql_type = BigInt)]
    pub other_transaction_version: i64,
    #[diesel(sql_type = BigInt)]
    pub other_change_index: i64,
    #[diesel(sql_type = Bool)]
    pub other_is_delete: bool,
    #[diesel(sql_type = Nullable<Jsonb>)]
    pub other_value: Option<serde_json::Value>,
    #[diesel(sql_type = Bool)]
    pub is_prev: bool
}

impl ChangeTableItemDiff {
    /// Creates a new `ChangeTableItemDiff` from transaction details
    pub fn from_transaction(
        current_transaction_version: i64,
        current_change_index: i64,
        table_handle: &str,
        key: serde_json::Value,
        other_transaction_version: i64,
        other_change_index: i64,
        other_is_delete: bool,
        other_value: Option<serde_json::Value>,
        is_prev: bool
    ) -> Self {
        ChangeTableItemDiff {
            current_transaction_version,
            current_change_index,
            table_handle: table_handle.to_string(),
            key: key.clone(),
            other_transaction_version,
            other_change_index,
            other_is_delete,
            other_value,
            is_prev
        }
    }

    pub fn into_change_table_item(
        self
    ) -> ChangeTableItem {
        let runner_id: i32 = env::var("RUNNER_ID")
        .unwrap_or("0".to_string()) // Default to "0" if the environment variable is not set
        .parse()
        .unwrap_or(0);
        match self.is_prev {
            true => ChangeTableItem { 
                transaction_version: self.current_transaction_version,
                transaction_block_height : 0,
                change_index: self.current_change_index,
                transaction_hash: "".to_string(),
                #[allow(deprecated)]
                transaction_timestamp: chrono::NaiveDateTime::from_timestamp(0, 0),
                transaction_sender: Some("".to_string()),
                transaction_entry_function_id_str: Some("".to_string()),
                is_transaction_success: false,
                state_key_hash: "".to_string(),
                is_delete: true,
                table_handle: self.table_handle,
                key_type: "".to_string(),
                key: self.key,
                value_type: None,
                value: None,
                prev_transaction_version: Some(self.other_transaction_version),
                prev_change_index: Some(self.other_change_index),
                prev_is_delete: Some(self.other_is_delete),
                prev_value: self.other_value,
                next_transaction_version: None,
                next_change_index: None,
                next_is_delete: None,
                next_value: None,
                runner_id: runner_id as i64
            },
            false => ChangeTableItem { 
                transaction_version: self.current_transaction_version,
                transaction_block_height : 0,
                change_index: self.current_change_index,
                transaction_hash: "".to_string(),
                #[allow(deprecated)]
                transaction_timestamp: chrono::NaiveDateTime::from_timestamp(0, 0),
                transaction_sender: Some("".to_string()),
                transaction_entry_function_id_str: Some("".to_string()),
                is_transaction_success: false,
                state_key_hash: "".to_string(),
                is_delete: true,
                table_handle: self.table_handle,
                key_type: "".to_string(),
                key: self.key,
                value_type: None,
                value: None,
                prev_transaction_version: None,
                prev_change_index: None,
                prev_is_delete: None,
                prev_value: None,
                next_transaction_version: Some(self.other_transaction_version),
                next_change_index: Some(self.other_change_index),
                next_is_delete: Some(self.other_is_delete),
                next_value: self.other_value,
                runner_id: runner_id as i64
            }
        }
    }

}



// Prevent conflicts with other things named `ChangeTableItem`
pub type ChangeTableItemModel = ChangeTableItem;

// Prevent conflicts with other things named `ChangeTableItemDiff`
pub type ChangeTableItemDiffModel = ChangeTableItemDiff;