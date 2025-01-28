// Copyright © Aptos Foundation

// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

// This is required because a diesel macro makes clippy sad
#![allow(clippy::extra_unused_lifetimes)]
#![allow(clippy::unused_unit)]
use field_count::FieldCount;
use serde::{Deserialize, Serialize};
use crate::schema::change_resources_partition;
use diesel::sql_types::{BigInt, Bool, Jsonb, Text, Nullable};
use std::env;

#[derive(Clone, Debug, Deserialize, Serialize, Queryable, FieldCount, Insertable)]
#[diesel(table_name = change_resources_partition)]
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
    pub prev_data: Option<serde_json::Value>,
    pub next_transaction_version: Option<i64>,
    pub next_change_index: Option<i64>,
    pub next_is_delete: Option<bool>,
    pub next_data: Option<serde_json::Value>,
    pub runner_id: i64
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
        let runner_id: i32 = env::var("RUNNER_ID")
        .unwrap_or("0".to_string()) // Default to "0" if the environment variable is not set
        .parse()
        .unwrap_or(0); 
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
            next_transaction_version: None,
            next_change_index: None,
            next_is_delete: None,
            next_data: None,
            runner_id: runner_id as i64
        }
    }
}

#[derive(Debug, QueryableByName, Clone, QueryId, Deserialize, Serialize)]

pub struct ChangeResourceDiff {
    #[diesel(sql_type = BigInt)]
    pub current_transaction_version: i64,
    #[diesel(sql_type = BigInt)]
    pub current_change_index: i64,
    #[diesel(sql_type = Text)]
    pub address: String,
    #[diesel(sql_type = Text)]
    pub resource_type: String,
    #[diesel(sql_type = BigInt)]
    pub other_transaction_version: i64,
    #[diesel(sql_type = BigInt)]
    pub other_change_index: i64,
    #[diesel(sql_type = Bool)]
    pub other_is_delete: bool,
    #[diesel(sql_type = Nullable<Jsonb>)]
    pub other_data: Option<serde_json::Value>,
    #[diesel(sql_type = Bool)]
    pub is_prev: bool
}

impl ChangeResourceDiff {
    /// Creates a new `ChangeResourceDiff` from transaction details
    pub fn from_transaction(
        current_transaction_version: i64,
        current_change_index: i64,
        address: &str,
        resource_type: &str,
        other_transaction_version: i64,
        other_change_index: i64,
        other_is_delete: bool,
        other_data: Option<serde_json::Value>,
        is_prev: bool
    ) -> Self {
        ChangeResourceDiff {
            current_transaction_version,
            current_change_index,
            address: address.to_string(),
            resource_type: resource_type.to_string(),
            other_transaction_version,
            other_change_index,
            other_is_delete,
            other_data,
            is_prev
        }
    }

        /// Converts `ChangeResourceDiff` to a `ChangeResource` by moving its values
    /// and setting default values for missing fields.
    pub fn into_change_resource(
        self
    ) -> ChangeResource {
        let runner_id: i32 = env::var("RUNNER_ID")
        .unwrap_or("0".to_string()) // Default to "0" if the environment variable is not set
        .parse()
        .unwrap_or(0); 
        match self.is_prev {
            true => ChangeResource {
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
                address: self.address,
                resource_type: self.resource_type,
                data: None,
                prev_transaction_version: Some(self.other_transaction_version),
                prev_change_index: Some(self.other_change_index),
                prev_is_delete: Some(self.other_is_delete),
                prev_data: self.other_data,
                next_transaction_version: None,
                next_change_index: None,
                next_is_delete: None,
                next_data: None,
                runner_id: runner_id as i64

            },
            false => ChangeResource {
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
                address: self.address,
                resource_type: self.resource_type,
                data: None,
                prev_transaction_version: None,
                prev_change_index: None,
                prev_is_delete: None,
                prev_data: None,
                next_transaction_version: Some(self.other_transaction_version),
                next_change_index: Some(self.other_change_index),
                next_is_delete: Some(self.other_is_delete),
                next_data: self.other_data,
                runner_id: runner_id as i64
            }
        }
    }
}


// Prevent conflicts with other things named `ChangeResource`
pub type ChangeResourceModel = ChangeResource;


// Prevent conflicts with other things named `ChangeResourceDiff`
pub type ChangeResourceDiffModel = ChangeResourceDiff;