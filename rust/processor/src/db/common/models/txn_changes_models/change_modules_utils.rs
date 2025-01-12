// Copyright © Aptos Foundation

// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

// This is required because a diesel macro makes clippy sad


#![allow(clippy::extra_unused_lifetimes)]
#![allow(clippy::unused_unit)]
use field_count::FieldCount;
use serde::{Deserialize, Serialize};
use crate::schema::change_modules;
use flate2::read::GzDecoder;
use std::io::Read;
use anyhow::Result;

#[derive(Debug, Deserialize)]
pub struct PackageRegistryResource {
    pub packages: Vec<Package>,
}

#[derive(Debug, Deserialize)]
pub struct Package {
    pub name: String,
    pub manifest: String,
    pub modules: Vec<Module>,
}

#[derive(Debug, Deserialize)]
pub struct Module {
    pub name: String,
    pub source: String,
    pub source_map: String
}

#[derive(Debug)]
pub struct ModuleInfo {
    pub package_name: String,
    pub manifest: String,
    pub module_name: String,
    pub source_code: String,
    pub source_map: String
}

// fn get_bytecode_metadata_from_hex(hex_str: &str) -> Result<(u32, Option<Vec<u8>>)> {
//     let bytecode = hex::decode(hex_str).map_err(|e| anyhow!("Failed to decode hex string: {}", e))?;
//     let module = CompiledModule::deserialize(&bytecode)
//         .map_err(|e| anyhow!("Failed to deserialize bytecode: {}", e))?;

//     let version = module.version;
//     let metadata = module.metadata.clone();

//     Ok((version, metadata))
// }

// /// Compile Move code from strings and return the compiled bytecode.
// fn compile_move_code_from_strings(
//     package_manifest: &str,
//     source_code: &str,
//     hex_str: &str, // Add hex string as a parameter
// ) -> Result<String> {
//     // Create a temporary directory
//     let temp_dir = temp_dir().join("move_compile_temp");
//     create_dir_all(&temp_dir).context("Failed to create temporary directory")?;

//     // Write the manifest (Move.toml) to the temp directory
//     let manifest_path = temp_dir.join("Move.toml");
//     fs::write(&manifest_path, package_manifest).context("Failed to write Move.toml")?;

//     // Create the sources directory and write the source code
//     let sources_dir = temp_dir.join("sources");
//     create_dir_all(&sources_dir).context("Failed to create sources directory")?;

//     let source_file_path = sources_dir.join("module.move");
//     fs::write(&source_file_path, source_code).context("Failed to write source code")?;

//     // Compile the package using move-package
//     let build_config = BuildConfig {
//         dev_mode: false,
//         test_mode: false,
//         ..Default::default()
//     };

//     let compiled_units = match build_config.compile_package(&temp_dir, &mut std::io::sink()) {
//         Ok(units) => units,
//         Err(e) => {
//             println!("Compilation failed: {}", e);
//             return Ok("".to_string());
//         }
//     };

//     // Extract bytecode version from hex string
//     let bytecode_version = match get_bytecode_version_from_hex(hex_str) {
//         Ok(version) => version,
//         Err(e) => {
//             println!("Failed to get bytecode version: {}", e);
//             return Ok("".to_string());
//         }
//     };

//     // Extract bytecode from the single compiled unit and convert to hex string
//     let unit = match compiled_units
//     .root_compiled_units
//     .into_iter()
//     .next() {
//         Some(unit) => unit,
//         None => {
//             println!("No compiled units found");
//             return Ok("".to_string());
//         }
//     };

//     let serialized = unit.unit.serialize(Some(bytecode_version));
//     let hex_string = hex::encode(serialized);

//     Ok(hex_string)
// }

pub fn decompress_gzip(data: &str) -> Result<String, std::io::Error> {
    let compressed_data = hex::decode(data.trim_start_matches("0x")).expect("Invalid hex");
    let mut decoder = GzDecoder::new(&compressed_data[..]);
    let mut decompressed_data = String::new();
    decoder.read_to_string(&mut decompressed_data)?;
    Ok(decompressed_data)
}

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
pub type PackageRegistryResourceModel = PackageRegistryResource;
pub type ModuleInfoModel = ModuleInfo;