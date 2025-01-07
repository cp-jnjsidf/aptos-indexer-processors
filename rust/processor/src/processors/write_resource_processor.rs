use super::{DefaultProcessingResult, ProcessorName, ProcessorTrait};
use crate::{
    db::common::models::write_changes_models::write_resource_utils::WriteResourceModel,
    gap_detectors::ProcessingResult,
    schema,
    utils::database::{execute_in_chunks, get_config_table_chunk_size, ArcDbPool},
};
use ahash::AHashMap;
use anyhow::bail;
use aptos_protos::transaction::v1::Transaction;
use async_trait::async_trait;
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


pub struct WriteResourceProcessor {
    connection_pool: ArcDbPool,
    per_table_chunk_sizes: AHashMap<String, usize>,
}

impl WriteResourceProcessor {
    pub fn new(connection_pool: ArcDbPool, per_table_chunk_sizes: AHashMap<String, usize>) -> Self {
        Self {
            connection_pool,
            per_table_chunk_sizes,
        }
    }
}

impl Debug for WriteResourceProcessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = &self.connection_pool.state();
        write!(
            f,
            "WriteResourceProcessor {{ connections: {:?}  idle_connections: {:?} }}",
            state.connections, state.idle_connections
        )
    }
}


async fn insert_to_db(
    conn: ArcDbPool,
    name: &'static str,
    start_version: u64,
    end_version: u64,
    write_resources: &[WriteResourceModel],
    per_table_chunk_sizes: &AHashMap<String, usize>,
) -> Result<(), diesel::result::Error> {
    tracing::trace!(
        name = name,
        start_version = start_version,
        end_version = end_version,
        "Inserting to db",
    );

    execute_in_chunks(
        conn,
        insert_write_resources_query,
        write_resources,
        get_config_table_chunk_size::<WriteResourceModel>("write_set_changes_table", per_table_chunk_sizes),
    )
    .await?;
    Ok(())
}

fn insert_write_resources_query(
    items_to_insert: Vec<WriteResourceModel>,
) -> (
    impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send,
    Option<&'static str>,
) {
    use schema::write_set_changes_table::dsl::*;
    (
        diesel::insert_into(schema::write_set_changes_table::table)
            .values(items_to_insert)
            .on_conflict((transaction_version, index))
            .do_update()
            .set((
                address.eq(excluded(address)),
                inserted_at.eq(excluded(inserted_at)),
            )),
        None,
    )
}

#[async_trait]
impl ProcessorTrait for WriteResourceProcessor {
    fn name(&self) -> &'static str {
        ProcessorName::WriteResourceProcessor.into()
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

        let mut write_resources = vec![];
        for txn in &transactions {
            let txn_version = txn.version as i64;
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

            let txn_hash = standardize_address(hex::encode(transaction_info.hash.as_slice()).as_str());
            for (wsc_index, wsc) in transaction_info.changes.iter().enumerate() {
                if let Change::WriteResource(wr) = wsc.change.as_ref().unwrap() {
                    let wr_data = serde_json::from_str::<Value>(wr.data.as_str()).unwrap_or(Value::Null);

                    let state_key_hash = standardize_address(
                        hex::encode(wr.state_key_hash.as_slice()).as_str(),
                    );
                    let resource_type = wr.type_str.clone();
                    let write_resource_address = standardize_address(&wr.address);
                    let write_resource = WriteResourceModel::from_transaction(
                        txn_version,
                        wsc_index as i64,
                        txn_hash.as_str(),
                        write_resource_address.as_str(),
                        state_key_hash.as_str(),
                        resource_type.as_str(),
                        wr_data,
                    );
                    write_resources.push(write_resource);



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
            &write_resources,
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
