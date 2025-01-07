// Copyright © Aptos Foundation

// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

// This is required because a diesel macro makes clippy sad
#![allow(clippy::extra_unused_lifetimes)]
#![allow(clippy::unused_unit)]
use field_count::FieldCount;
use serde::{Deserialize, Serialize};


#[derive(Clone, Debug, Deserialize, Serialize, Queryable, FieldCount, Insertable)]
#[diesel(table_name = crate::schema::write_set_changes_table)]
pub struct WriteResource {
    pub transaction_version: i64,
    pub index: i64,
    pub hash: String,
    pub address: String,
    pub state_key_hash: String,
    pub resource_type: String,
    pub data: serde_json::Value,
}

impl WriteResource {
    pub fn from_transaction(
        transaction_version: i64,
        index: i64,
        hash: &str,
        address: &str,
        state_key_hash: &str,
        resource_type: &str,
        data: serde_json::Value,
    ) -> Self {
        WriteResource {
            transaction_version,
            index,
            hash: hash.to_string(),
            address: address.to_string(),
            state_key_hash: state_key_hash.to_string(),
            resource_type: resource_type.to_string(),
            data,
        }
    }
}

// Prevent conflicts with other things named `WriteResource`
pub type WriteResourceModel = WriteResource;