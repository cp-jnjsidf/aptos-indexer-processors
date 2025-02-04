use super::{DefaultProcessingResult, ProcessorName, ProcessorTrait};
use crate::{
    db::common::models::txn_changes_models::change_resource_utils::ChangeResourceModel,
    db::common::models::txn_changes_models::change_table_item_utils::ChangeTableItemModel,
    gap_detectors::ProcessingResult,
    utils::database::{execute_in_chunks, get_config_table_chunk_size, ArcDbPool},
    schema
};
use ahash::AHashMap;
use anyhow::bail;
use aptos_protos::transaction::v1::Transaction;
use async_trait::async_trait;
use chrono::NaiveDateTime;
use diesel::{
    pg::{upsert::excluded, Pg},
    query_builder::QueryFragment,
    ExpressionMethods,
};
use std::fmt::Debug;
use tracing::error;

use crate::utils::util::standardize_address;
use aptos_protos::transaction::v1::write_set_change::Change;
use serde_json::Value;
#[allow(unused_imports)]
use anyhow::Context;



pub struct TxnChangesProcessor {
    connection_pool: ArcDbPool,
    per_table_chunk_sizes: AHashMap<String, usize>,
}

impl TxnChangesProcessor {
    pub fn new(connection_pool: ArcDbPool, per_table_chunk_sizes: AHashMap<String, usize>) -> Self {
        Self {
            connection_pool,
            per_table_chunk_sizes,
        }
    }
}

impl Debug for TxnChangesProcessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = &self.connection_pool.state();
        write!(
            f,
            "TxnChangesProcessor {{ connections: {:?}  idle_connections: {:?} }}",
            state.connections, state.idle_connections
        )
    }
}


async fn insert_to_db(
    conn: ArcDbPool,
    name: &'static str,
    start_version: u64,
    end_version: u64,
    change_resources: &[ChangeResourceModel],
    change_table_items: &[ChangeTableItemModel],

    per_table_chunk_sizes: &AHashMap<String, usize>,
) -> Result<(), diesel::result::Error> {
    tracing::trace!(
        name = name,
        start_version = start_version,
        end_version = end_version,
        "Inserting to db",
    );

    execute_in_chunks(
        conn.clone(),
        insert_change_resources_query,
        change_resources,
        get_config_table_chunk_size::<ChangeResourceModel>("change_resources", per_table_chunk_sizes),
    )
    .await?;

    execute_in_chunks(
        conn,
        insert_table_items_query,
        change_table_items,
        get_config_table_chunk_size::<ChangeTableItemModel>("change_table_items", per_table_chunk_sizes),
    )
    .await?;
    Ok(())
}

fn insert_change_resources_query(
    items_to_insert: Vec<ChangeResourceModel>,
) -> (
    impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send,
    Option<&'static str>,
) {
    use schema::change_resources_partition::dsl::*;
    (
        diesel::insert_into(schema::change_resources_partition::table)
            .values(items_to_insert)
            .on_conflict((transaction_version, change_index, runner_id))
            .do_update()
            .set((
                address.eq(excluded(address)),
                inserted_at.eq(excluded(inserted_at)),
            )),
        None,
    )
}

fn insert_table_items_query(
    items_to_insert: Vec<ChangeTableItemModel>,
) -> (
    impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send,
    Option<&'static str>,
) {
    use schema::change_table_items_partition::dsl::*;
    (
        diesel::insert_into(schema::change_table_items_partition::table)
            .values(items_to_insert)
            .on_conflict((transaction_version, change_index, runner_id))
            .do_update()
            .set((
                inserted_at.eq(excluded(inserted_at)),
            )),
        None,
    )
}

fn sanitize_json(value: &mut Value) {
    match value {
        Value::String(s) => {
            // Remove all `\0` characters from the string
            *s = s.replace('\0', "");
        }
        Value::Object(map) => {
            // Recursively sanitize all values in the map
            for v in map.values_mut() {
                sanitize_json(v);
            }
        }
        Value::Array(arr) => {
            // Recursively sanitize all items in the array
            for v in arr.iter_mut() {
                sanitize_json(v);
            }
        }
        _ => {
            // Do nothing for non-string primitive values (Number, Bool, Null)
        }
    }
}

fn parse_change_data(resource_data: &str) -> Option<Value> {
    if let Ok(mut json) = serde_json::from_str::<Value>(resource_data) {
        if let Some(inner) = json.as_str() {
            let mut res = serde_json::from_str::<Value>(inner).unwrap_or(json);
            sanitize_json(&mut res);
            return Some(res);
            
        }
        sanitize_json(&mut json);
        return Some(json);
    }
    tracing::warn!("Skipped parse_change_data for data: {}", resource_data);
    None
}


#[async_trait]
impl ProcessorTrait for TxnChangesProcessor {
    fn name(&self) -> &'static str {
        ProcessorName::TxnChangesProcessor.into()
    }

    async fn process_transactions(
        &self,
        transactions: Vec<Transaction>,
        start_version: u64,
        end_version: u64,
        _: Option<u64>,
    ) -> anyhow::Result<ProcessingResult> {
        let processing_start = std::time::Instant::now();
        let last_transaction_timestamp = transactions.last().unwrap().timestamp.clone();

        let mut change_resources = vec![];
        let mut change_table_items = vec![];
        for txn in &transactions {
            let transaction_version = txn.version as i64;
            let transaction_info = match txn.info.as_ref() {
                Some(info) => info,
                None => {
                    tracing::warn!(
                        transaction_version = txn.version,
                        "Skipping transaction due to missing info"
                    );
                    continue;
                }
            };
            let txn_timestamp_int = txn
            .timestamp
            .as_ref()
            .expect("Transaction timestamp doesn't exist!")
            .seconds;
            #[allow(deprecated)]
            let txn_timestamp =
            NaiveDateTime::from_timestamp_opt(txn_timestamp_int, 0).expect("Txn Timestamp is invalid!");

            for (change_index, wsc) in transaction_info.changes.iter().enumerate() {
                if let Some(change) = wsc.change.as_ref() {
                    match change{
                        // Handle WriteResource
                        Change::WriteResource(resource) =>
                        {
                            let resource_type = resource.type_str.clone();
                            let resouce_address = standardize_address(&resource.address);
                            let resource_parsed = ChangeResourceModel::from_transaction(
                                transaction_version,
                                change_index as i64,
                                txn_timestamp,
                                resouce_address.as_str(),
                                resource_type.as_str()
                            );
                            change_resources.push(resource_parsed);   
                        }
                        // Handle DeleteResource
                        Change::DeleteResource(resource) =>
                        {
                            let resource_type = resource.type_str.clone();
                            let resouce_address = standardize_address(&resource.address);
                            let resource_parsed = ChangeResourceModel::from_transaction(
                                transaction_version,
                                change_index as i64,
                                txn_timestamp,
                                resouce_address.as_str(),
                                resource_type.as_str(),
                            );
                            change_resources.push(resource_parsed);   
                        }
                        // Handle WriteTableItem
                        Change::WriteTableItem(table_item) =>
                        {
                            if table_item.data.is_none(){
                                error!(
                                    "Skipping table item, the data is None for write: transaction_version: {}, change_index: {}",
                                    transaction_version, change_index
                                );
                                continue;
                            }

                            let table_item_data  = table_item.data.as_ref().unwrap();
                            let key_data = parse_change_data(table_item_data.key.as_str());
                            if key_data.is_none() {
                                error!(
                                    "Skipping table item due to parsing error with the key_data of a write action: transaction_version: {}, change_index: {}, raw data: {}",
                                    transaction_version, change_index, table_item_data.key.as_str()
                                );
                                continue;
                            }

                            let key_data = key_data.unwrap();
                        
                           
                            let table_handle_address = standardize_address(&table_item.handle);
                            let table_item_parsed = ChangeTableItemModel::from_transaction(
                                transaction_version,
                                change_index as i64,
                                txn_timestamp,
                                table_handle_address.as_str(),
                                key_data,
                            );
                            change_table_items.push(table_item_parsed);   
                        }
                        Change::DeleteTableItem(table_item) =>
                        {
                            if table_item.data.is_none(){
                                error!(
                                    "Skipping table item, the data is None for delete: transaction_version: {}, change_index: {}",
                                    transaction_version, change_index
                                );
                                continue;
                            }

                            let table_item_data  = table_item.data.as_ref().unwrap();
                            let key_data = parse_change_data(table_item_data.key.as_str());
                            if key_data.is_none() {
                                error!(
                                    "Skipping table item due to parsing error with the key_data of a delete action: transaction_version: {}, change_index: {}, raw data: {}",
                                    transaction_version, change_index, table_item_data.key.as_str()
                                );
                                continue;
                            }

                            let key_data = key_data.unwrap();

                            let table_handle_address = standardize_address(&table_item.handle);
                            let table_item_parsed = ChangeTableItemModel::from_transaction(
                                transaction_version,
                                change_index as i64,
                                txn_timestamp,
                                table_handle_address.as_str(),
                                key_data,
                            );
                            change_table_items.push(table_item_parsed);   
                        }
                        _ => {}
                    }
                }
            }

    }

        let processing_duration_in_secs = processing_start.elapsed().as_secs_f64();
        let db_insertion_start = std::time::Instant::now();

        let tx_result = insert_to_db(
            self.get_pool(),
            self.name(),
            start_version,
            end_version,
            &change_resources,
            &change_table_items,
            &self.per_table_chunk_sizes,
        )
        .await;

        let db_insertion_duration_in_secs = db_insertion_start.elapsed().as_secs_f64();
        match tx_result {
            Ok(_) => Ok(ProcessingResult::DefaultProcessingResult(
                DefaultProcessingResult {
                    start_version,
                    end_version,
                    processing_duration_in_secs,
                    db_insertion_duration_in_secs,
                    last_transaction_timestamp,
                },
            )),
            Err(e) => {
                error!(
                    start_version = start_version,
                    end_version = end_version,
                    processor_name = self.name(),
                    error = ?e,
                    "[Processor] Error inserting transactions to db",
                );
                bail!(e)
            },
        }
    }

    fn connection_pool(&self) -> &ArcDbPool {
        &self.connection_pool
    }
}
