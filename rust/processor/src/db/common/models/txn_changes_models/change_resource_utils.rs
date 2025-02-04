// Copyright © Aptos Foundation

// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

// This is required because a diesel macro makes clippy sad
#![allow(clippy::extra_unused_lifetimes)]
#![allow(clippy::unused_unit)]
use field_count::FieldCount;
use serde::{Deserialize, Serialize};
use crate::schema::change_resources_partition;
use diesel::sql_types::{BigInt, Text, Nullable};
use std::env;

#[derive(Clone, Debug, Deserialize, Serialize, Queryable, FieldCount, Insertable)]
#[diesel(table_name = change_resources_partition)]
pub struct ChangeResource {
    pub transaction_version: i64,
    pub change_index: i64,
    pub transaction_timestamp: chrono::NaiveDateTime,
    pub address: String,
    pub resource_type: String,
    pub prev_transaction_version: Option<i64>,
    pub prev_change_index: Option<i64>,
    pub next_transaction_version: Option<i64>,
    pub next_change_index: Option<i64>,
    pub runner_id: i64
}

impl ChangeResource {
    pub fn from_transaction(
        transaction_version: i64,
        change_index: i64,
        transaction_timestamp: chrono::NaiveDateTime,
        address: &str,
        resource_type: &str,
    ) -> Self {
        let runner_id: i32 = env::var("RUNNER_ID")
        .unwrap_or("0".to_string()) // Default to "0" if the environment variable is not set
        .parse()
        .unwrap_or(0); 
        ChangeResource {
            transaction_version,
            change_index,
            transaction_timestamp,
            address: address.to_string(),
            resource_type: resource_type.to_string(),
            prev_transaction_version: None,
            prev_change_index: None,
            next_transaction_version: None,
            next_change_index: None,
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
    #[diesel(sql_type = Nullable<BigInt>)]
    pub prev_transaction_version: Option<i64>,
    #[diesel(sql_type = Nullable<BigInt>)]
    pub prev_change_index: Option<i64>,
    #[diesel(sql_type = Nullable<BigInt>)]
    pub next_transaction_version: Option<i64>,
    #[diesel(sql_type = Nullable<BigInt>)]
    pub next_change_index: Option<i64>,
}

impl ChangeResourceDiff {
    /// Creates a new `ChangeResourceDiff` from transaction details
    pub fn from_transaction(
        current_transaction_version: i64,
        current_change_index: i64,
        address: &str,
        resource_type: &str,
        prev_transaction_version: Option<i64>,
        prev_change_index: Option<i64>,
        next_transaction_version: Option<i64>,
        next_change_index: Option<i64>,
    ) -> Self {
        ChangeResourceDiff {
            current_transaction_version,
            current_change_index,
            address: address.to_string(),
            resource_type: resource_type.to_string(),
            prev_transaction_version,
            prev_change_index,
            next_transaction_version,
            next_change_index
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
        ChangeResource {
                transaction_version: self.current_transaction_version,
                change_index: self.current_change_index,
                #[allow(deprecated)]
                transaction_timestamp: chrono::NaiveDateTime::from_timestamp(0, 0),
                address: self.address,
                resource_type: self.resource_type,
                prev_transaction_version: self.prev_transaction_version,
                prev_change_index: self.prev_change_index,
                next_transaction_version: self.next_transaction_version,
                next_change_index: self.next_change_index,
                runner_id: runner_id as i64

        }
    }
}


// Prevent conflicts with other things named `ChangeResource`
pub type ChangeResourceModel = ChangeResource;


// Prevent conflicts with other things named `ChangeResourceDiff`
pub type ChangeResourceDiffModel = ChangeResourceDiff;