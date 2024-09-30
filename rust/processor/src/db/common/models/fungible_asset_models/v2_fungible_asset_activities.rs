// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

// This is required because a diesel macro makes clippy sad
#![allow(clippy::extra_unused_lifetimes)]
#![allow(clippy::unused_unit)]

use super::v2_fungible_asset_utils::{FeeStatement, FungibleAssetEvent};
use crate::{
    db::common::models::{
        coin_models::{
            coin_activities::CoinActivity,
            coin_utils::{CoinEvent, CoinInfoType, EventGuidResource},
        },
        object_models::v2_object_utils::ObjectAggregatedDataMapping,
        token_v2_models::v2_token_utils::TokenStandard,
    },
    schema::fungible_asset_activities,
    utils::util::standardize_address,
};
use ahash::AHashMap;
use anyhow::Context;
use aptos_protos::transaction::v1::{Event, TransactionInfo, UserTransactionRequest};
use bigdecimal::{BigDecimal, Zero};
use field_count::FieldCount;
use serde::{Deserialize, Serialize};

pub const GAS_FEE_EVENT: &str = "0x1::aptos_coin::GasFeeEvent";
// We will never have a negative number on chain so this will avoid collision in postgres
pub const BURN_GAS_EVENT_CREATION_NUM: i64 = -1;
pub const BURN_GAS_EVENT_INDEX: i64 = -1;

pub type OwnerAddress = String;
pub type CoinType = String;
// Primary key of the current_coin_balances table, i.e. (owner_address, coin_type)
pub type CurrentCoinBalancePK = (OwnerAddress, CoinType);
pub type EventToCoinType = AHashMap<EventGuidResource, CoinType>;

#[derive(Clone, Debug, Deserialize, FieldCount, Identifiable, Insertable, Serialize)]
#[diesel(primary_key(transaction_version, event_index))]
#[diesel(table_name = fungible_asset_activities)]
pub struct FungibleAssetActivity {
    pub transaction_version: i64,
    pub event_index: i64,
    pub owner_address: Option<String>,
    pub storage_id: String,
    pub asset_type: Option<String>,
    pub is_frozen: Option<bool>,
    pub amount: Option<BigDecimal>,
    pub number_used_gas_units : Option<BigDecimal>,
    pub max_gas_price : Option<BigDecimal>,
    pub type_: String,
    pub is_gas_fee: bool,
    pub gas_fee_payer_address: Option<String>,
    pub is_transaction_success: bool,
    pub entry_function_id_str: Option<String>,
    pub block_height: i64,
    pub token_standard: String,
    pub transaction_timestamp: chrono::NaiveDateTime,
    pub storage_refund_amount: BigDecimal,
    pub sender : Option<String>,
    pub txn_hash: String,
    pub txn_args: Option<serde_json::Value>,
    pub txn_timestamp_id : i64,
}

impl FungibleAssetActivity {
    pub fn get_v2_from_event(
        event: &Event,
        txn_version: i64,
        block_height: i64,
        txn_timestamp: chrono::NaiveDateTime,
        event_index: i64,
        entry_function_id_str: &Option<String>,
        object_aggregated_data_mapping: &ObjectAggregatedDataMapping,
        txn_hash : &String,
        sender : &Option<String>,
        txn_args: &Option<serde_json::Value>,
        txn_timestamp_id : i64,
    ) -> anyhow::Result<Option<Self>> {
        let event_type = event.type_str.clone();
        if let Some(fa_event) =
            &FungibleAssetEvent::from_event(event_type.as_str(), &event.data, txn_version)?
        {
            let (storage_id, is_frozen, amount) = match fa_event {
                FungibleAssetEvent::WithdrawEvent(inner) => (
                    standardize_address(&event.key.as_ref().unwrap().account_address),
                    None,
                    Some(inner.amount.clone()),
                ),
                FungibleAssetEvent::DepositEvent(inner) => (
                    standardize_address(&event.key.as_ref().unwrap().account_address),
                    None,
                    Some(inner.amount.clone()),
                ),
                FungibleAssetEvent::FrozenEvent(inner) => (
                    standardize_address(&event.key.as_ref().unwrap().account_address),
                    Some(inner.frozen),
                    None,
                ),
                FungibleAssetEvent::WithdrawEventV2(inner) => (
                    standardize_address(&inner.store),
                    None,
                    Some(inner.amount.clone()),
                ),
                FungibleAssetEvent::DepositEventV2(inner) => (
                    standardize_address(&inner.store),
                    None,
                    Some(inner.amount.clone()),
                ),
                FungibleAssetEvent::FrozenEventV2(inner) => {
                    (standardize_address(&inner.store), Some(inner.frozen), None)
                },
            };

            // The event account address will also help us find fungible store which tells us where to find
            // the metadata
            let maybe_object_metadata = object_aggregated_data_mapping.get(&storage_id);
            // The ObjectCore might not exist in the transaction if the object got deleted
            let maybe_owner_address = maybe_object_metadata
                .map(|metadata| &metadata.object.object_core)
                .map(|object_core| object_core.get_owner_address());
            // The FungibleStore might not exist in the transaction if it's a secondary store that got burnt
            let maybe_asset_type = maybe_object_metadata
                .and_then(|metadata| metadata.fungible_asset_store.as_ref())
                .map(|fa| fa.metadata.get_reference_address());

            return Ok(Some(Self {
                transaction_version: txn_version,
                event_index,
                owner_address: maybe_owner_address,
                storage_id: storage_id.clone(),
                asset_type: maybe_asset_type,
                is_frozen,
                amount,
                number_used_gas_units: None,
                max_gas_price: None,
                type_: event_type.clone(),
                is_gas_fee: false,
                gas_fee_payer_address: None,
                is_transaction_success: true,
                entry_function_id_str: entry_function_id_str.clone(),
                block_height,
                token_standard: TokenStandard::V2.to_string(),
                transaction_timestamp: txn_timestamp,
                sender: sender.clone(),
                txn_hash: txn_hash.to_string(),
                txn_args: txn_args.clone(),
                storage_refund_amount: BigDecimal::zero(),
                txn_timestamp_id,
            }));
        }
        Ok(None)
    }

    pub fn get_v1_from_event(
        event: &Event,
        txn_version: i64,
        block_height: i64,
        transaction_timestamp: chrono::NaiveDateTime,
        entry_function_id_str: &Option<String>,
        event_to_coin_type: &EventToCoinType,
        event_index: i64,
        txn_hash : &String,
        sender : &Option<String>,
        txn_args: &Option<serde_json::Value>,
        txn_timestamp_id : i64,
    ) -> anyhow::Result<Option<Self>> {
        if let Some(inner) =
            CoinEvent::from_event(event.type_str.as_str(), &event.data, txn_version)?
        {
            let (owner_address, amount, coin_type_option) = match inner {
                CoinEvent::WithdrawCoinEvent(inner) => (
                    standardize_address(&event.key.as_ref().unwrap().account_address),
                    inner.amount.clone(),
                    None,
                ),
                CoinEvent::DepositCoinEvent(inner) => (
                    standardize_address(&event.key.as_ref().unwrap().account_address),
                    inner.amount.clone(),
                    None,
                ),
            };
            let coin_type = if let Some(coin_type) = coin_type_option {
                coin_type
            } else {
                let event_key = event.key.as_ref().context("event must have a key")?;
                let event_move_guid = EventGuidResource {
                    addr: standardize_address(event_key.account_address.as_str()),
                    creation_num: event_key.creation_number as i64,
                };
                // Given this mapping only contains coin type < 1000 length, we should not assume that the mapping exists.
                // If it doesn't exist, skip.
                match event_to_coin_type.get(&event_move_guid) {
                    Some(coin_type) => coin_type.clone(),
                    None => {
                        tracing::warn!(
                        "Could not find event in resources (CoinStore), version: {}, event guid: {:?}, mapping: {:?}",
                        txn_version, event_move_guid, event_to_coin_type
                    );
                        return Ok(None);
                    },
                }
            };

            let storage_id =
                CoinInfoType::get_storage_id(coin_type.as_str(), owner_address.as_str());

            Ok(Some(Self {
                transaction_version: txn_version,
                event_index,
                owner_address: Some(owner_address),
                storage_id,
                asset_type: Some(coin_type),
                is_frozen: None,
                amount: Some(amount),
                number_used_gas_units: None,
                max_gas_price: None,
                type_: event.type_str.clone(),
                is_gas_fee: false,
                gas_fee_payer_address: None,
                is_transaction_success: true,
                entry_function_id_str: entry_function_id_str.clone(),
                block_height,
                token_standard: TokenStandard::V1.to_string(),
                transaction_timestamp,
                storage_refund_amount: BigDecimal::zero(),
                sender: sender.clone(),
                txn_hash: txn_hash.clone(),
                txn_args: txn_args.clone(),
                txn_timestamp_id,
            }))
        } else {
            Ok(None)
        }
    }

    /// Artificially creates a gas event. If it's a fee payer, still show gas event to the sender
    /// but with an extra field to indicate the fee payer.
    pub fn get_gas_event(
        txn_info: &TransactionInfo,
        user_transaction_request: &UserTransactionRequest,
        entry_function_id_str: &Option<String>,
        transaction_version: i64,
        transaction_timestamp: chrono::NaiveDateTime,
        block_height: i64,
        fee_statement: Option<FeeStatement>,
        txn_args: &Option<serde_json::Value>,
        txn_timestamp_id : i64,
    ) -> Self {
        let txn_hash = standardize_address(hex::encode(txn_info.hash.as_slice()).as_str());
        let sender = standardize_address(&user_transaction_request.sender);
        let v1_activity = CoinActivity::get_gas_event(
            txn_info,
            user_transaction_request,
            entry_function_id_str,
            transaction_version,
            transaction_timestamp,
            block_height,
            fee_statement,
        );
        let storage_id = CoinInfoType::get_storage_id(
            v1_activity.coin_type.as_str(),
            v1_activity.owner_address.as_str(),
        );
        Self {
            transaction_version,
            event_index: v1_activity.event_index.unwrap(),
            owner_address: Some(v1_activity.owner_address),
            storage_id,
            asset_type: Some(v1_activity.coin_type),
            is_frozen: None,
            amount: Some(v1_activity.amount),
            number_used_gas_units: Some(BigDecimal::from(txn_info.gas_used)),
            max_gas_price: Some(BigDecimal::from(user_transaction_request.max_gas_amount)),
            type_: v1_activity.activity_type,
            is_gas_fee: v1_activity.is_gas_fee,
            gas_fee_payer_address: v1_activity.gas_fee_payer_address,
            is_transaction_success: v1_activity.is_transaction_success,
            entry_function_id_str: v1_activity.entry_function_id_str,
            block_height,
            token_standard: TokenStandard::V1.to_string(),
            transaction_timestamp,
            storage_refund_amount: v1_activity.storage_refund_amount,
            txn_timestamp_id,
            txn_hash,
            sender: Some(sender),
            txn_args: txn_args.clone(),
        }
    }
}
