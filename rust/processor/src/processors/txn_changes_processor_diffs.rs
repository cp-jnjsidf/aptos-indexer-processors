use super::{DefaultProcessingResult, ProcessorName, ProcessorTrait};
use crate::{
    db::common::models::txn_changes_models::change_resource_utils::{ChangeResourceDiffModel, ChangeResourceModel, ChangeResourceOldDataQueryModel},
    db::common::models::txn_changes_models::change_table_item_utils::{ChangeTableItemDiffModel, ChangeTableItemModel, ChangeTableItemOldDataQueryModel},
    gap_detectors::ProcessingResult,
    utils::database::{execute_in_chunks, get_config_table_chunk_size, ArcDbPool},
    schema
};
use ahash::AHashMap;
use anyhow::bail;
use aptos_protos::transaction::v1::Transaction;
use async_trait::async_trait;
use diesel::{
    pg::Pg,
    query_builder::QueryFragment, pg::upsert::excluded,
};
use diesel_async::RunQueryDsl;
use std::fmt::Debug;
use tracing::error;

use crate::utils::util::standardize_address;
use aptos_protos::transaction::v1::write_set_change::Change;
use serde_json::Value;
#[allow(unused_imports)]
use anyhow::Context;



pub struct TxnChangesProcessorDiffs {
    connection_pool: ArcDbPool,
    per_table_chunk_sizes: AHashMap<String, usize>,
}

impl TxnChangesProcessorDiffs {
    pub fn new(connection_pool: ArcDbPool, per_table_chunk_sizes: AHashMap<String, usize>) -> Self {
        Self {
            connection_pool,
            per_table_chunk_sizes,
        }
    }
}

impl Debug for TxnChangesProcessorDiffs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = &self.connection_pool.state();
        write!(
            f,
            "TxnChangesProcessorDiffs {{ connections: {:?}  idle_connections: {:?} }}",
            state.connections, state.idle_connections
        )
    }
}


async fn insert_to_db(
    conn: ArcDbPool,
    name: &'static str,
    start_version: u64,
    end_version: u64,
    change_resources_diffs:  &[ChangeResourceDiffModel],
    change_table_items_diffs: &[ChangeTableItemDiffModel],

    per_table_chunk_sizes: &AHashMap<String, usize>,
) -> Result<(), diesel::result::Error> {
    tracing::trace!(
        name = name,
        start_version = start_version,
        end_version = end_version,
        "Updating previous versions in db",
    );

    execute_in_chunks(
        conn.clone(),
        insert_change_resources_diff_query,
        change_resources_diffs,
        get_config_table_chunk_size::<ChangeResourceModel>("change_resources", per_table_chunk_sizes),
    )
    .await?;

    execute_in_chunks(
        conn,
        insert_table_items_diffs_query,
        change_table_items_diffs,
        get_config_table_chunk_size::<ChangeTableItemModel>("change_table_items", per_table_chunk_sizes),
    )
    .await?;
    Ok(())
}

fn insert_change_resources_diff_query(
    diffs: Vec<ChangeResourceDiffModel>,
) -> (
    impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send,
    Option<&'static str>,
) {
    use schema::change_resources::dsl::*;
    use diesel::prelude::*;
    let full_diffs: Vec<ChangeResourceModel> = diffs.into_iter()
        .map(ChangeResourceDiffModel::into_change_resource)
        .collect();
    (
        diesel::insert_into(change_resources)
            .values(full_diffs) // Insert all the diffs
            .on_conflict((transaction_version, change_index)) // Conflict key
            .do_update()
            .set((
                prev_transaction_version.eq(excluded(prev_transaction_version)),
                prev_change_index.eq(excluded(prev_change_index)),
                prev_is_delete.eq(excluded(prev_is_delete)),
                prev_data.eq(excluded(prev_data)),
                transaction_block_height.eq(transaction_block_height),
                transaction_hash.eq(transaction_hash),
                transaction_timestamp.eq(transaction_timestamp),
                transaction_sender.eq(transaction_sender),
                transaction_entry_function_id_str.eq(transaction_entry_function_id_str),
                is_transaction_success.eq(is_transaction_success),
                state_key_hash.eq(state_key_hash),
                is_delete.eq(is_delete),
                address.eq(address),
                resource_type.eq(resource_type),
                data.eq(data),
                inserted_at.eq(inserted_at),
            )),
        None,
    )
}

fn insert_table_items_diffs_query(
    diffs: Vec<ChangeTableItemDiffModel>,
) -> (
    impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send,
    Option<&'static str>,
) {
    use schema::change_table_items::dsl::*;
    use diesel::prelude::*;
    let full_diffs: Vec<ChangeTableItemModel> = diffs.into_iter()
    .map(ChangeTableItemDiffModel::into_change_table_item)
    .collect();
    (
        diesel::insert_into(change_table_items)
            .values(full_diffs) // Insert all the diffs
            .on_conflict((transaction_version, change_index)) // Conflict key
            .do_update()
            .set((
                prev_transaction_version.eq(excluded(prev_transaction_version)),
                prev_change_index.eq(excluded(prev_change_index)),
                prev_is_delete.eq(excluded(prev_is_delete)),
                prev_value.eq(excluded(prev_value)),
                transaction_block_height.eq(transaction_block_height),
                transaction_hash.eq(transaction_hash),
                transaction_timestamp.eq(transaction_timestamp),
                transaction_sender.eq(transaction_sender),
                transaction_entry_function_id_str.eq(transaction_entry_function_id_str),
                is_transaction_success.eq(is_transaction_success),
                state_key_hash.eq(state_key_hash),
                is_delete.eq(is_delete),
                table_handle.eq(table_handle),
                key_type.eq(key_type),
                key.eq(key),
                value_type.eq(value_type),
                value.eq(value),
                inserted_at.eq(inserted_at),
            )),
        None,
    )
}

use diesel::sql_types::{Text, BigInt};
use diesel::deserialize::QueryableByName;
use std::sync::Arc;
use diesel::query_builder::QueryId;
use diesel_async::AsyncPgConnection;
async fn execute_in_chunks_with_values<T, U, F, V>(
    pool: ArcDbPool,
    build_query: F,
    items: &[T],
    chunk_size: usize,
) -> Result<Vec<U>, diesel::result::Error>
where
    U: QueryableByName<Pg> + Send + Sync + 'static + Clone,
    T: Sync + Send + Clone + 'static,
    F: Fn(&[T]) -> V + Send + Sync + 'static + Clone,
    V: QueryFragment<Pg> + QueryId + diesel_async::methods::LoadQuery<'static, AsyncPgConnection, U> + Send + 'static,
{
    // Split items into chunks
    let chunks: Vec<_> = items.chunks(chunk_size).map(|chunk| chunk.to_vec()).collect();

    // Spawn tasks to process each chunk concurrently
    let tasks = chunks.into_iter().map(|chunk| {
        let pool = Arc::clone(&pool);
        let build_query = build_query.clone();

        async move {
            // Get a connection from the pool
            let conn = match pool.get().await {
                Ok(conn) => conn,
                Err(e) => {
                    tracing::warn!("Failed to get connection: {:?}", e);
                    return Vec::new(); // Return an empty vector on connection error
                }
            };
            let mut conn = conn;

            // Build the query
            let query = build_query(&chunk);

            // Attempt to execute the query
            match query.get_results::<U>(&mut conn).await {
                Ok(results) => results,
                Err(e) => {
                    tracing::warn!("Failed to execute query: {:?}", e);
                    Vec::new() // Return an empty vector on query execution error
                }
            }
        }
    });

    // Collect results from all tasks
    let mut results = Vec::new();
    for chunk_results in futures::future::join_all(tasks).await {
        results.extend(chunk_results); // Extend results with the chunk results
    }

    Ok(results)
}


pub async fn get_prev_resources_in_batch(
    conn: ArcDbPool,
    items: Vec<(String, String, i64)>, // address, resource_type, max_version
    per_table_chunk_sizes: &AHashMap<String, usize>
) -> Result<AHashMap<(String, String, i64), ChangeResourceOldDataQueryModel>, diesel::result::Error> {
    let mut result_map = AHashMap::new();

    // Use `execute_in_chunks_with_values` to split the items into chunks and execute the query for each chunk
    let results: Vec<ChangeResourceOldDataQueryModel> = execute_in_chunks_with_values(
        conn,
        |chunk| {
            // Clone the chunk into separate vectors to avoid borrowing issues
            let address_vec: Vec<_> = chunk.iter().map(|(addr, _, _)| addr.clone()).collect();
            let resource_type_vec: Vec<_> = chunk.iter().map(|(_, res_type, _)| res_type.clone()).collect();
            let max_version_vec: Vec<_> = chunk.iter().map(|(_, _, max_ver)| *max_ver).collect();
            diesel::sql_query(
                "WITH input_pairs (address, resource_type, max_version) AS (
                    SELECT * FROM UNNEST($1::text[], $2::text[], $3::bigint[])
                )
                SELECT
                    ip.max_version as current_transaction_version, 
                    ip.address as address, 
                    ip.resource_type as resource_type, 
                    cr.transaction_version as prev_transaction_version, 
                    cr.change_index as prev_change_index, 
                    cr.is_delete as prev_is_delete, 
                    cr.data as prev_data
                FROM input_pairs ip
                INNER JOIN LATERAL (
                    SELECT cr.transaction_version, cr.change_index, cr.is_delete, cr.data
                    FROM change_resources cr
                    WHERE cr.address = ip.address
                      AND cr.resource_type = ip.resource_type
                      AND cr.transaction_version < ip.max_version
                    ORDER BY cr.transaction_version DESC, cr.change_index DESC
                    LIMIT 1
                ) cr ON true"
            )
                .bind::<diesel::sql_types::Array<Text>, _>(address_vec)
                .bind::<diesel::sql_types::Array<Text>, _>(resource_type_vec)
                .bind::<diesel::sql_types::Array<BigInt>, _>(max_version_vec)
        },
        &items,
        get_config_table_chunk_size::<ChangeResourceModel>("change_resources", per_table_chunk_sizes),
    )
    .await?;

    // Populate result_map with query results
    for result in results {
        result_map.insert(
            (
                result.address.clone(),
                result.resource_type.clone(),
                result.current_transaction_version,
            ),
            result,
        );
    }

    Ok(result_map)
}

pub async fn get_prev_table_items_in_batch(
    conn: ArcDbPool,
    items: Vec<(String, Value, i64)>, // table_handle, key, max_version
    per_table_chunk_sizes: &AHashMap<String, usize>
) -> Result<AHashMap<(String, Value, i64), ChangeTableItemOldDataQueryModel>, diesel::result::Error> {
    let mut result_map = AHashMap::new();

    // Use execute_in_chunks_with_values to query in batches
    let results: Vec<ChangeTableItemOldDataQueryModel> = execute_in_chunks_with_values(
        conn,
        |chunk| {
            // Clone the chunk into separate vectors to avoid borrowing issues
            let table_handle_vec: Vec<_> = chunk.iter().map(|(table_handle, _, _)| table_handle.clone()).collect();
            let key_vec: Vec<_> = chunk.iter().map(|(_, key, _)| key.clone()).collect();
            let max_version_vec: Vec<_> = chunk.iter().map(|(_, _, max_ver)| *max_ver).collect();
            diesel::sql_query(
                "WITH input_pairs (table_handle, key, max_version) AS (
                    SELECT * FROM UNNEST($1::text[], $2::jsonb[], $3::bigint[])
                    )
                    SELECT
                    ip.max_version as current_transaction_version, 
                    ip.table_handle as table_handle, 
                    ip.key as key, 
                    cti.transaction_version as prev_transaction_version, 
                    cti.change_index as prev_change_index, 
                    cti.is_delete as prev_is_delete, 
                    cti.value as prev_value
                    FROM input_pairs ip
                    INNER JOIN LATERAL (
                        SELECT cti.transaction_version, cti.change_index, cti.is_delete, cti.value
                        FROM change_table_items cti
                        WHERE cti.table_handle = ip.table_handle
                        AND cti.key = ip.key
                        AND cti.transaction_version < ip.max_version
                        ORDER BY cti.transaction_version DESC, cti.change_index DESC
                        LIMIT 1
                    ) cti ON true"
            )
            .bind::<diesel::sql_types::Array<Text>, _>(table_handle_vec)
            .bind::<diesel::sql_types::Array<diesel::sql_types::Jsonb>, _>(key_vec)
            .bind::<diesel::sql_types::Array<BigInt>, _>(max_version_vec)
        },
        &items,
        get_config_table_chunk_size::<ChangeTableItemModel>("change_table_items", per_table_chunk_sizes),
    )
    .await?;
    // Populate result_map with query results
    for result in results {
        result_map.insert(
            (
                result.table_handle.clone(),
                result.key.clone(),
                result.current_transaction_version,
            ),
            result,
        );
    }

    Ok(result_map)
}


fn parse_change_data(resource_data: &str) -> Option<Value> {
    if let Ok(json) = serde_json::from_str::<Value>(resource_data) {
        if let Some(inner) = json.as_str() {
            return serde_json::from_str::<Value>(inner).ok().or(Some(json));
        }
        return Some(json);
    }
    None
}



#[async_trait]
impl ProcessorTrait for TxnChangesProcessorDiffs {
    fn name(&self) -> &'static str {
        ProcessorName::TxnChangesProcessorDiffs.into()
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

        let mut change_resources_diffs = vec![];
        let mut change_table_items_diffs = vec![];

        let mut address_resource_timestamp_triplete  = vec![];
        let mut table_handle_key_timestamp_triplete = vec![];
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
            for (change_index, wsc) in transaction_info.changes.iter().enumerate() {
                if let Some(change) = wsc.change.as_ref() {
                    match change{
                        // Handle WriteResource
                        Change::WriteResource(resource) => 
                        {
                            address_resource_timestamp_triplete.push((
                                standardize_address(&resource.address),
                                resource.type_str.clone(),
                                transaction_version,
                            ));
                        }
                        // Handle DeleteResource
                        Change::DeleteResource(resource) =>
                        {
                            address_resource_timestamp_triplete.push((
                                standardize_address(&resource.address),
                                resource.type_str.clone(),
                                transaction_version,
                            ));   
                        }
                        // Handle WriteTableItem
                        Change::WriteTableItem(table_item) =>
                        { 
                            let table_item_data  = table_item.data.as_ref().unwrap();
                            let key_data = parse_change_data(table_item_data.key.as_str());
                            if key_data.is_none() {
                                error!(
                                    "Skipping table item diff due to parsing error with the key_data of a write action: transaction_version: {}, change_index: {}, raw data: {}",
                                    transaction_version, change_index, table_item_data.key.as_str()
                                );
                                continue;
                            }

                            let key_data = key_data.unwrap();

                            table_handle_key_timestamp_triplete.push((
                                standardize_address(&table_item.handle),
                                key_data,
                                transaction_version,
                            ));
                        }
                        Change::DeleteTableItem(table_item) =>
                        {
                            let table_item_data  = table_item.data.as_ref().unwrap();
                            let key_data = parse_change_data(table_item_data.key.as_str());
                            if key_data.is_none() {
                                error!(
                                    "Skipping table item diff due to parsing error with the key_data of a write action: transaction_version: {}, change_index: {}, raw data: {}",
                                    transaction_version, change_index, table_item_data.key.as_str()
                                );
                                continue;
                            }

                            let key_data = key_data.unwrap();

                            table_handle_key_timestamp_triplete.push((
                                standardize_address(&table_item.handle),
                                key_data,
                                transaction_version,
                            ));
                        }
                        _ => {}
                    }
                }
            }
        }

        // Fetch previous states for resources and table items in batch
        let prev_resources_map = get_prev_resources_in_batch(
            self.get_pool(),
            address_resource_timestamp_triplete.clone(),
            &self.per_table_chunk_sizes,
        )
        .await?;

        let prev_table_items_map = get_prev_table_items_in_batch(
            self.get_pool(),
            table_handle_key_timestamp_triplete.clone(),
            &self.per_table_chunk_sizes,
        )
        .await?;
        // Process transactions and build diffs in one loop
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

            for (change_index, wsc) in transaction_info.changes.iter().enumerate() {
                if let Some(change) = wsc.change.as_ref() {
                    match change {
                        // Handle WriteResource
                        Change::WriteResource(resource) => {
                            if let Some(prev) = prev_resources_map.get(&(
                                standardize_address(&resource.address),
                                resource.type_str.clone(),
                                transaction_version,
                            )) {
                                change_resources_diffs.push(ChangeResourceDiffModel {
                                    transaction_version,
                                    change_index: change_index as i64,
                                    address : standardize_address(&resource.address),
                                    resource_type : resource.type_str.clone(),
                                    prev_transaction_version: Some(prev.prev_transaction_version),
                                    prev_change_index: Some(prev.prev_change_index),
                                    prev_is_delete: Some(prev.prev_is_delete),
                                    prev_data: prev.prev_data.clone(),
                                });
                            }
                        }

                        // Handle DeleteResource
                        Change::DeleteResource(resource) => {
                            if let Some(prev) = prev_resources_map.get(&(
                                standardize_address(&resource.address),
                                resource.type_str.clone(),
                                transaction_version,
                            )) {
                                change_resources_diffs.push(ChangeResourceDiffModel {
                                    transaction_version,
                                    change_index: change_index as i64,
                                    address : standardize_address(&resource.address),
                                    resource_type : resource.type_str.clone(),
                                    prev_transaction_version: Some(prev.prev_transaction_version),
                                    prev_change_index: Some(prev.prev_change_index),
                                    prev_is_delete: Some(prev.prev_is_delete),
                                    prev_data: prev.prev_data.clone(),
                                });
                            }
                        }

                        // Handle WriteTableItem
                        Change::WriteTableItem(table_item) => {
                            if let Some(prev) = prev_table_items_map.get(&(
                                standardize_address(&table_item.handle),
                                parse_change_data(&table_item.data.as_ref().unwrap().key).unwrap(),
                                transaction_version,
                            )) {
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
                            
                                change_table_items_diffs.push(ChangeTableItemDiffModel {
                                    transaction_version,
                                    change_index: change_index as i64,
                                    table_handle: standardize_address(&table_item.handle),
                                    key: key_data,
                                    prev_transaction_version: Some(prev.prev_transaction_version),
                                    prev_change_index: Some(prev.prev_change_index),
                                    prev_is_delete: Some(prev.prev_is_delete),
                                    prev_value: prev.prev_value.clone(),
                                });
                            }
                        }

                        // Handle DeleteTableItem
                        Change::DeleteTableItem(table_item) => {
                            if let Some(prev) = prev_table_items_map.get(&(
                                standardize_address(&table_item.handle),
                                parse_change_data(&table_item.data.as_ref().unwrap().key).unwrap(),
                                transaction_version,
                            )) {
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
                                change_table_items_diffs.push(ChangeTableItemDiffModel {
                                    transaction_version,
                                    change_index: change_index as i64,
                                    table_handle: standardize_address(&table_item.handle),
                                    key: key_data,
                                    prev_transaction_version: Some(prev.prev_transaction_version),
                                    prev_change_index: Some(prev.prev_change_index),
                                    prev_is_delete: Some(prev.prev_is_delete),
                                    prev_value: prev.prev_value.clone(),
                                });
                            }
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
            &change_resources_diffs,
            &change_table_items_diffs,
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
