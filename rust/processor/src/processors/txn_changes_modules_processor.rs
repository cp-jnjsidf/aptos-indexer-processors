use super::{DefaultProcessingResult, ProcessorName, ProcessorTrait};
use crate::{
    db::common::models::txn_changes_models::change_modules_utils::ChangeModulesModel,
    gap_detectors::ProcessingResult,
    utils::database::{execute_in_chunks, get_config_table_chunk_size, ArcDbPool},
    schema
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

use crate::utils::util::{standardize_address, get_entry_function_from_user_request};
use aptos_protos::transaction::v1::write_set_change::Change;
use aptos_protos::transaction::v1::transaction::TxnData;
use chrono::NaiveDateTime;



pub struct TxnChangesModulesProcessor {
    connection_pool: ArcDbPool,
    per_table_chunk_sizes: AHashMap<String, usize>,
}

impl TxnChangesModulesProcessor {
    pub fn new(connection_pool: ArcDbPool, per_table_chunk_sizes: AHashMap<String, usize>) -> Self {
        Self {
            connection_pool,
            per_table_chunk_sizes,
        }
    }
}

impl Debug for TxnChangesModulesProcessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = &self.connection_pool.state();
        write!(
            f,
            "TxnChangesModulesProcessor {{ connections: {:?}  idle_connections: {:?} }}",
            state.connections, state.idle_connections
        )
    }
}


async fn insert_to_db(
    conn: ArcDbPool,
    name: &'static str,
    start_version: u64,
    end_version: u64,
    change_modules: &[ChangeModulesModel],

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
        insert_change_modules_query,
        change_modules,
        get_config_table_chunk_size::<ChangeModulesModel>("change_modules", per_table_chunk_sizes),
    )
    .await?;

    Ok(())
}

fn insert_change_modules_query(
    items_to_insert: Vec<ChangeModulesModel>,
) -> (
    impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send,
    Option<&'static str>,
) {
    use schema::change_modules::dsl::*;
    (
        diesel::insert_into(schema::change_modules::table)
            .values(items_to_insert)
            .on_conflict((transaction_version, change_index))
            .do_update()
            .set((
                address.eq(excluded(address)),
                inserted_at.eq(excluded(inserted_at)),
            )),
        None,
    )
}


#[async_trait]
impl ProcessorTrait for TxnChangesModulesProcessor {
    fn name(&self) -> &'static str {
        ProcessorName::TxnChangesModulesProcessor.into()
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

        let mut change_modules = vec![];
        for txn in &transactions {
            let transaction_version = txn.version as i64;
            let block_height = txn.block_height as i64;
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
            let txn_data = txn.txn_data.as_ref().unwrap();
            let txn_timestamp_int = txn
            .timestamp
            .as_ref()
            .expect("Transaction timestamp doesn't exist!")
            .seconds;
            #[allow(deprecated)]
            let txn_timestamp =
            NaiveDateTime::from_timestamp_opt(txn_timestamp_int, 0).expect("Txn Timestamp is invalid!");
            let (sender, entry_function_id_str) = match txn_data {
                TxnData::User(tx_inner) => {
                    let user_request = tx_inner
                        .request
                        .as_ref()
                        .expect("Sends is not present in user txn");
                    let sender = Some(standardize_address(&user_request.sender));
                    let entry_function_id_str = get_entry_function_from_user_request(user_request);
                    (sender, entry_function_id_str)
                }
                _ => (None, None),
                
            };

            let is_transaction_success = transaction_info.success;
            for (change_index, wsc) in transaction_info.changes.iter().enumerate() {
                if let Some(change) = wsc.change.as_ref() {
                    match change{
                        // Handle WriteModule
                        // Change::WriteModule(module_obj) =>
                        // {
                        //     let is_delete = false;
                            
                        //     let state_key_hash = standardize_address(
                        //         hex::encode(module_obj.state_key_hash.as_slice()).as_str(),
                        //     );
                        //     let module_obj_address = standardize_address(&module_obj.address);
                        //     let resource_parsed = ChangeModulesModel::from_transaction(
                        //         transaction_version,
                        //         block_height,
                        //         change_index as i64,
                        //         txn_hash.as_str(),
                        //         txn_timestamp,
                        //         &sender,
                        //         &entry_function_id_str,
                        //         is_transaction_success,
                        //         state_key_hash.as_str(),
                        //         is_delete,
                        //         address.as_str(),
                        //         name.as_str(),
                        //         &bytecode,
                        //         &abi,
                        //         &package_manifest,
                        //         &source_code,
                        //         &is_source_correct   
                        //     );
                        //     change_modules.push(resource_parsed);   
                        // }
                        // Handle DeleteModule
                        Change::DeleteModule(module_obj) =>
                        {
                            let is_delete = true;
                            let state_key_hash = standardize_address(
                                hex::encode(module_obj.state_key_hash.as_slice()).as_str(),
                            );
                            if module_obj.module.is_none(){
                                error!(
                                    "Skipping module change, the module is None for delete: transaction_version: {}, change_index: {}",
                                    transaction_version, change_index
                                );
                                continue;
                            }
                            let module_obj_data  = module_obj.module.as_ref().unwrap();
                            let name = module_obj_data.name.clone();
                            let address = standardize_address(&module_obj.address);
                            // if module_obj.data.is_none(){
                            //     error!(
                            //         "Skipping module change, the module is None for delete: transaction_version: {}, change_index: {}",
                            //         transaction_version, change_index
                            //     );
                            //     continue;
                            // }
                            // let module_obj_data  = module_obj.data.as_ref().unwrap();
                            // let abi_obj  = module_obj_data.abi.as_ref().unwrap();
                            // let name = abi_obj.name.clone();
                            // let address = standardize_address(&module_obj.address);
                            let bytecode = None;
                            let package_manifest = None;
                            let source_code = None;
                            let is_source_correct = None;
                            let abi = Some(serde_json::Value::Null);
                            let resource_parsed = ChangeModulesModel::from_transaction(
                                transaction_version,
                                block_height,
                                change_index as i64,
                                txn_hash.as_str(),
                                txn_timestamp,
                                &sender,
                                &entry_function_id_str,
                                is_transaction_success,
                                state_key_hash.as_str(),
                                is_delete,
                                address.as_str(),
                                name.as_str(),
                                &bytecode,
                                &abi,
                                &package_manifest,
                                &source_code,
                                &is_source_correct   
                            );
                            change_modules.push(resource_parsed);   
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
            &change_modules,
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
