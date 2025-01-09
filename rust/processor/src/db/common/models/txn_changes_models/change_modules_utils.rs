// Copyright © Aptos Foundation

// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

// This is required because a diesel macro makes clippy sad
#![allow(clippy::extra_unused_lifetimes)]
#![allow(clippy::unused_unit)]
use field_count::FieldCount;
use serde::{Deserialize, Serialize};
use crate::schema::change_modules;


#[derive(Clone, Debug, Deserialize, Serialize, Queryable, FieldCount, Insertable)]
#[diesel(table_name = change_modules)]
pub struct ChangeModule {
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
    pub name: String,
    pub bytecode: Option<String>,
    pub abi: Option<serde_json::Value>,
    pub package_manifest: Option<String>,
    pub source_code: Option<String>,
    pub is_source_correct:  Option<bool>
}

impl ChangeModule {
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
        name: &str,
        bytecode: &Option<String>,
        abi: &Option<serde_json::Value>,
        package_manifest: &Option<String>,
        source_code: &Option<String>,
        is_source_correct: &Option<bool>,
    ) -> Self {
        ChangeModule {
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
            name: name.to_string(),
            bytecode: bytecode.clone(),
            abi: abi.clone(),
            package_manifest: package_manifest.clone(),
            source_code: source_code.clone(),
            is_source_correct: is_source_correct.clone()
        }
    }
}

// Prevent conflicts with other things named `ChangeModule`
pub type ChangeModulesModel = ChangeModule;