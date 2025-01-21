// Copyright © Aptos Foundation

// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

// This is required because a diesel macro makes clippy sad
#![allow(clippy::extra_unused_lifetimes)]
#![allow(clippy::unused_unit)]
use field_count::FieldCount;
use serde::{Deserialize, Serialize};
use crate::schema::change_resources;
use diesel::sql_types::{BigInt, Bool, Jsonb, Text, Nullable};

#[derive(Clone, Debug, Deserialize, Serialize, Queryable, FieldCount, Insertable)]
#[diesel(table_name = change_resources)]
pub struct ChangeResource {
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
    pub address: String,
    pub resource_type: String,
    pub data: Option<serde_json::Value>,
    pub prev_transaction_version: Option<i64>,
    pub prev_change_index: Option<i64>,
    pub prev_is_delete: Option<bool>,
    pub prev_data: Option<serde_json::Value>
}

impl ChangeResource {
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
        address: &str,
        resource_type: &str,
        data: &Option<serde_json::Value>,
    ) -> Self {
        ChangeResource {
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
            address: address.to_string(),
            resource_type: resource_type.to_string(),
            data: data.clone(),
            prev_transaction_version: None,
            prev_change_index: None,
            prev_is_delete: None,
            prev_data: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Insertable, QueryableByName)]
#[diesel(table_name = change_resources)]
pub struct ChangeResourceDiff {
    pub transaction_version: i64,
    pub change_index: i64,
    pub address: String,
    pub resource_type: String,
    pub prev_transaction_version: Option<i64>,
    pub prev_change_index: Option<i64>,
    pub prev_is_delete: Option<bool>,
    pub prev_data: Option<serde_json::Value>,
}

impl ChangeResourceDiff {
    /// Creates a new `ChangeResourceDiff` from transaction details
    pub fn from_transaction(
        transaction_version: i64,
        change_index: i64,
        address: &str,
        resource_type: &str,
        prev_transaction_version: Option<i64>,
        prev_change_index: Option<i64>,
        prev_is_delete: Option<bool>,
        prev_data: Option<serde_json::Value>,
    ) -> Self {
        ChangeResourceDiff {
            transaction_version,
            change_index,
            address: address.to_string(),
            resource_type: resource_type.to_string(),
            prev_transaction_version,
            prev_change_index,
            prev_is_delete,
            prev_data,
        }
    }

        /// Converts `ChangeResourceDiff` to a `ChangeResource` by moving its values
    /// and setting default values for missing fields.
    pub fn into_change_resource(
        self
    ) -> ChangeResource {
        ChangeResource {
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
            address: self.address,
            resource_type: self.resource_type,
            data: None,
            prev_transaction_version: self.prev_transaction_version,
            prev_change_index: self.prev_change_index,
            prev_is_delete: self.prev_is_delete,
            prev_data: self.prev_data,
        }
    }
}



#[derive(Debug, QueryableByName, Clone, QueryId)]
pub struct ChangeResourceOldDataQuery {
    #[diesel(sql_type = BigInt)]
    pub current_transaction_version: i64,

    #[diesel(sql_type = Text)]
    pub address: String,

    #[diesel(sql_type = Text)]
    pub resource_type: String,

    #[diesel(sql_type = BigInt)]
    pub prev_transaction_version: i64,

    #[diesel(sql_type = BigInt)]
    pub prev_change_index: i64,

    #[diesel(sql_type = Bool)]
    pub prev_is_delete: bool,

    #[diesel(sql_type = Nullable<Jsonb>)]
    pub prev_data: Option<serde_json::Value>,
}



// Prevent conflicts with other things named `ChangeResource`
pub type ChangeResourceModel = ChangeResource;


// Prevent conflicts with other things named `ChangeResourceDiff`
pub type ChangeResourceDiffModel = ChangeResourceDiff;

pub type ChangeResourceOldDataQueryModel = ChangeResourceOldDataQuery;