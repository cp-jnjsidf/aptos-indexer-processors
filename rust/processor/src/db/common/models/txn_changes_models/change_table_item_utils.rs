// Copyright © Aptos Foundation

// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

// This is required because a diesel macro makes clippy sad
#![allow(clippy::extra_unused_lifetimes)]
#![allow(clippy::unused_unit)]
use field_count::FieldCount;
use serde::{Deserialize, Serialize};
use crate::schema::change_table_items;
use diesel::sql_types::{BigInt, Bool, Jsonb, Text, Nullable};


#[derive(Clone, Debug, Deserialize, Serialize, Queryable, FieldCount, Insertable)]
#[diesel(table_name = change_table_items)]
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
            prev_value : None
        }
    }
}

#[derive(Debug, QueryableByName, Clone)]
pub struct ChangeTableItemOldDataQuery {
    #[diesel(sql_type = BigInt)]
    pub current_transaction_version: i64,

    #[diesel(sql_type = Text)]
    pub table_handle: String,

    #[diesel(sql_type = Jsonb)]
    pub key: serde_json::Value,

    #[diesel(sql_type = BigInt)]
    pub prev_transaction_version: i64,

    #[diesel(sql_type = BigInt)]
    pub prev_change_index: i64,

    #[diesel(sql_type = Bool)]
    pub prev_is_delete: bool,

    #[diesel(sql_type = Nullable<Jsonb>)]
    pub prev_value: Option<serde_json::Value>,
}
#[derive(Clone, Debug, Deserialize, Serialize, Insertable, QueryableByName)]
#[diesel(table_name = change_table_items)]
pub struct ChangeTableItemDiff {
    pub transaction_version: i64,
    pub change_index: i64,
    pub table_handle: String,
    pub key: serde_json::Value,
    pub prev_transaction_version: Option<i64>,
    pub prev_change_index: Option<i64>,
    pub prev_is_delete: Option<bool>,
    pub prev_value: Option<serde_json::Value>,
}

impl ChangeTableItemDiff {
    /// Creates a new `ChangeTableItemDiff` from transaction details
    pub fn from_transaction(
        transaction_version: i64,
        change_index: i64,
        table_handle: &str,
        key: serde_json::Value,
        prev_transaction_version: Option<i64>,
        prev_change_index: Option<i64>,
        prev_is_delete: Option<bool>,
        prev_value: Option<serde_json::Value>,
    ) -> Self {
        ChangeTableItemDiff {
            transaction_version,
            change_index,
            table_handle: table_handle.to_string(),
            key: key.clone(),
            prev_transaction_version,
            prev_change_index,
            prev_is_delete,
            prev_value,
        }
    }

    pub fn into_change_table_item(
        self
    ) -> ChangeTableItem {
        ChangeTableItem {
            transaction_version: self.transaction_version,
            transaction_block_height : 0,
            change_index: self.change_index,
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
            prev_transaction_version: self.prev_transaction_version,
            prev_change_index: self.prev_change_index,
            prev_is_delete: self.prev_is_delete,
            prev_value: self.prev_value,
        }
    }

}



// Prevent conflicts with other things named `ChangeTableItem`
pub type ChangeTableItemModel = ChangeTableItem;

// Prevent conflicts with other things named `ChangeTableItemDiff`
pub type ChangeTableItemDiffModel = ChangeTableItemDiff;

pub type ChangeTableItemOldDataQueryModel = ChangeTableItemOldDataQuery;