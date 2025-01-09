// Copyright © Aptos Foundation

// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

// This is required because a diesel macro makes clippy sad
#![allow(clippy::extra_unused_lifetimes)]
#![allow(clippy::unused_unit)]
use field_count::FieldCount;
use serde::{Deserialize, Serialize};
use crate::schema::change_resources;


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
    // pub generic_type_params: Option<serde_json::Value>
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
        data: &Option<serde_json::Value>//,
        // generic_type_params: &Option<serde_json::Value>,
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
            data: data.clone()//,
            // generic_type_params: generic_type_params.clone(),
        }
    }
}

// Prevent conflicts with other things named `ChangeResource`
pub type ChangeResourceModel = ChangeResource;