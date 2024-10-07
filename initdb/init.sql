CREATE TABLE IF NOT EXISTS tx_processed_cache (
    id SERIAL PRIMARY KEY,
    last_processed_version BIGINT,
    last_insertion timestamp DEFAULT now() NOT NULL,
    runner_id int4,
    upper_bound BIGINT
);

CREATE SCHEMA IF NOT EXISTS custom_proccesor AUTHORIZATION postgres;
CREATE TABLE IF NOT EXISTS custom_proccesor.custom_event (
	insertion_timestamp int8 NOT NULL,
	tx_from varchar NOT NULL,
	tx_to varchar NOT NULL,
	tx_hash varchar NOT NULL,
	tx_block_number int8 NOT NULL,
	tx_method_id varchar NOT NULL,
	tx_args jsonb NOT NULL,
	tx_timestamp int8 NOT NULL,
	tx_version bigserial NOT NULL,
	tx_gas int8 NOT NULL,
	tx_gas_used int8 NOT NULL,
	tx_gas_price int8 NOT NULL,
	tx_moving_coins_logs jsonb NOT NULL,
	"final" bool NOT NULL,
	CONSTRAINT custom_event_pkey PRIMARY KEY (tx_version)
);
CREATE TABLE IF NOT EXISTS custom_proccesor.missing_references (
	account_address varchar NOT NULL,
	creation_number int8 NOT NULL,
	coin_module_address varchar NOT NULL,
	tx_version int8 NOT NULL,
	"type" varchar NOT NULL,
	amount_not_normalized int8 NOT NULL,
	balance_not_normalized int8 NOT NULL,
	CONSTRAINT missing_references_pkey PRIMARY KEY (account_address, creation_number, coin_module_address, tx_version)
);
CREATE INDEX IF NOT EXISTS idx_account_address_creation_number_coin_module_address ON custom_proccesor.missing_references USING btree (account_address, creation_number, coin_module_address);

CREATE TABLE IF NOT EXISTS public.fungible_asset_activities (
	transaction_version int8 NOT NULL,
	event_index int8 NOT NULL,
	owner_address varchar(66) NULL,
	storage_id varchar(66) NOT NULL,
	asset_type varchar(1000) NULL,
	is_frozen bool NULL,
	amount numeric NULL,
	number_used_gas_units numeric NULL,
	max_gas_price numeric NULL,
	"type" varchar NOT NULL,
	is_gas_fee bool NOT NULL,
	gas_fee_payer_address varchar(66) NULL,
	is_transaction_success bool NOT NULL,
	entry_function_id_str varchar(1000) NULL,
	block_height int8 NOT NULL,
	token_standard varchar(10) NOT NULL,
	transaction_timestamp timestamp NOT NULL,
	inserted_at timestamp DEFAULT now() NOT NULL,
	txn_hash varchar(66) NOT NULL,
	sender varchar(66) NULL,
	txn_args jsonb NULL,
	txn_timestamp_id int8 NOT NULL,
	storage_refund_amount numeric NOT NULL,
	balance numeric NULL,
	CONSTRAINT fungible_asset_activities_pkey2 PRIMARY KEY (transaction_version, event_index)
);

CREATE TABLE IF NOT EXISTS public.processor_status (
	processor varchar(50) NOT NULL,
	last_success_version int8 NOT NULL,
	last_updated timestamp DEFAULT now() NOT NULL,
	last_transaction_timestamp timestamp NULL,
	CONSTRAINT processor_status_pkey PRIMARY KEY (processor)
);


BEGIN;

DO $$
DECLARE
    should_insert_data BOOLEAN := FALSE;  -- Set to TRUE to allow inserts, can change to FALSE to skip
BEGIN
    IF should_insert_data THEN
        -- Fungible asset activities inserts
        INSERT INTO public.fungible_asset_activities (transaction_version, event_index, owner_address, storage_id, asset_type, is_frozen, amount, number_used_gas_units, max_gas_price, type, is_gas_fee, gas_fee_payer_address, is_transaction_success, entry_function_id_str, block_height, token_standard, transaction_timestamp, inserted_at, txn_hash, sender, txn_args, txn_timestamp_id, storage_refund_amount) 
        VALUES 
        (1520040, 2, '0x2f74eede1e19256c38f9cdc53a2bd235da637415d54ab1c03f3d1d080d713ccc', '0xa86d9f2129403981382831cec37e636c1816826d7eba85bf9d5fc18d3e3101d4', '0x1::aptos_coin::AptosCoin', NULL, 0, NULL, NULL, '0x1::coin::DepositEvent', FALSE, NULL, TRUE, '0x1::aptos_account::transfer', 757699, 'v1', '2024-10-01 00:48:32', '2024-10-01 00:48:32', '0x73351680d43f24b51cbeac27ec4e4a663b162cf649c3c5a4ae31ec97d08610c6', '0x33ad4416bb194f8f70274a831df97928e1bb61b8e21ac669cbe494052ba8d292', '["0x2f74eede1e19256c38f9cdc53a2bd235da637415d54ab1c03f3d1d080d713ccc", "0"]', 1727780000000.0, 0),
        (1520043, -1, '0x6186f0782b5d8cc2c1d7def221675764c28a0c95dbc89d0e756c9c0eb309a864', '0x2b955cc4a0c4b3a84a0be03ec2576989b4a11dda88b303c6c43e610fcfd9f3d6', '0x1::aptos_coin::AptosCoin', NULL, 187800, 1878.0, 2000.0, '0x1::aptos_coin::GasFeeEvent', TRUE, NULL, TRUE, '0x1::aptos_account::transfer', 757700, 'v1', '2024-10-01 00:48:33', '2024-10-01 00:48:33', '0xaa8cab62dc327ae3a215c4a68e502f5f9e1b1a1aee1d01eb5ea0fd2c515e45f6', '0x6186f0782b5d8cc2c1d7def221675764c28a0c95dbc89d0e756c9c0eb309a864', '["0x2f7b0a47d85f05e513002339d8c626f1769b9f8c1cd98cf9ec7bbf08c128f4cf", "0"]', 1727780000000.0, 0),
        (1520043, 1, '0x6186f0782b5d8cc2c1d7def221675764c28a0c95dbc89d0e756c9c0eb309a864', '0x2b955cc4a0c4b3a84a0be03ec2576989b4a11dda88b303c6c43e610fcfd9f3d6', '0x1::aptos_coin::AptosCoin', NULL, 0, NULL, NULL, '0x1::coin::WithdrawEvent', FALSE, NULL, TRUE, '0x1::aptos_account::transfer', 757700, 'v1', '2024-10-01 00:48:33', '2024-10-01 00:48:33', '0xaa8cab62dc327ae3a215c4a68e502f5f9e1b1a1aee1d01eb5ea0fd2c515e45f6', '0x6186f0782b5d8cc2c1d7def221675764c28a0c95dbc89d0e756c9c0eb309a864', '["0x2f7b0a47d85f05e513002339d8c626f1769b9f8c1cd98cf9ec7bbf08c128f4cf", "0"]', 1727780000000.0, 0),
        (1520043, 2, '0x2f7b0a47d85f05e513002339d8c626f1769b9f8c1cd98cf9ec7bbf08c128f4cf', '0xe48a54a43920f914fcd0fa8a306a52e34176428c366bf16bb5d3de5efeabfb5e', '0x1::aptos_coin::AptosCoin', NULL, 0, NULL, NULL, '0x1::coin::DepositEvent', FALSE, NULL, TRUE, '0x1::aptos_account::transfer', 757700, 'v1', '2024-10-01 00:48:33', '2024-10-01 00:48:33', '0xaa8cab62dc327ae3a215c4a68e502f5f9e1b1a1aee1d01eb5ea0fd2c515e45f6', '0x6186f0782b5d8cc2c1d7def221675764c28a0c95dbc89d0e756c9c0eb309a864', '["0x2f7b0a47d85f05e513002339d8c626f1769b9f8c1cd98cf9ec7bbf08c128f4cf", "0"]', 1727780000000.0, 0);

        -- Processor status inserts
        INSERT INTO public.processor_status (processor, last_success_version, last_updated, last_transaction_timestamp) 
        VALUES 
        ('fungible_asset_processor_nano', 2770999, '2024-10-01 10:50:07.952', '2022-10-19 07:01:40.438'),
        ('user_transaction_processor', 1159987999, '2024-10-01 11:24:31.247', '2024-08-14 04:27:23.993'),
        ('fungible_asset_processor_old', 2997999, '2024-09-30 18:04:44.350', '2022-10-19 08:57:23.360'),
        ('fungible_asset_processor_old_micro', 2961999, '2024-10-01 09:57:01.854', '2022-10-19 08:42:15.171'),
        ('fungible_asset_processor', 209852397, '2024-10-02 08:29:21.953', '2023-08-05 01:01:07.711');
    END IF;
END $$;

COMMIT;





-- INSERT INTO public.fungible_asset_activities (transaction_version, event_index, owner_address, storage_id, asset_type, is_frozen, amount, number_used_gas_units, max_gas_price, type, is_gas_fee, gas_fee_payer_address, is_transaction_success, entry_function_id_str, block_height, token_standard, transaction_timestamp, inserted_at, txn_hash, sender, txn_args, txn_timestamp_id, storage_refund_amount) 
-- VALUES (1520040, 2, '0x2f74eede1e19256c38f9cdc53a2bd235da637415d54ab1c03f3d1d080d713ccc', '0xa86d9f2129403981382831cec37e636c1816826d7eba85bf9d5fc18d3e3101d4', '0x1::aptos_coin::AptosCoin', NULL, 0, NULL, NULL, '0x1::coin::DepositEvent', False, NULL, True, '0x1::aptos_account::transfer', 757699, 'v1', '2024-10-01 00:48:32', '2024-10-01 00:48:32', '0x73351680d43f24b51cbeac27ec4e4a663b162cf649c3c5a4ae31ec97d08610c6', '0x33ad4416bb194f8f70274a831df97928e1bb61b8e21ac669cbe494052ba8d292', '["0x2f74eede1e19256c38f9cdc53a2bd235da637415d54ab1c03f3d1d080d713ccc", "0"]', 1727780000000.0, 0);

-- INSERT INTO public.fungible_asset_activities (transaction_version, event_index, owner_address, storage_id, asset_type, is_frozen, amount, number_used_gas_units, max_gas_price, type, is_gas_fee, gas_fee_payer_address, is_transaction_success, entry_function_id_str, block_height, token_standard, transaction_timestamp, inserted_at, txn_hash, sender, txn_args, txn_timestamp_id, storage_refund_amount) 
-- VALUES (1520043, -1, '0x6186f0782b5d8cc2c1d7def221675764c28a0c95dbc89d0e756c9c0eb309a864', '0x2b955cc4a0c4b3a84a0be03ec2576989b4a11dda88b303c6c43e610fcfd9f3d6', '0x1::aptos_coin::AptosCoin', NULL, 187800, 1878.0, 2000.0, '0x1::aptos_coin::GasFeeEvent', True, NULL, True, '0x1::aptos_account::transfer', 757700, 'v1', '2024-10-01 00:48:33', '2024-10-01 00:48:33', '0xaa8cab62dc327ae3a215c4a68e502f5f9e1b1a1aee1d01eb5ea0fd2c515e45f6', '0x6186f0782b5d8cc2c1d7def221675764c28a0c95dbc89d0e756c9c0eb309a864', '["0x2f7b0a47d85f05e513002339d8c626f1769b9f8c1cd98cf9ec7bbf08c128f4cf", "0"]', 1727780000000.0, 0);

-- INSERT INTO public.fungible_asset_activities (transaction_version, event_index, owner_address, storage_id, asset_type, is_frozen, amount, number_used_gas_units, max_gas_price, type, is_gas_fee, gas_fee_payer_address, is_transaction_success, entry_function_id_str, block_height, token_standard, transaction_timestamp, inserted_at, txn_hash, sender, txn_args, txn_timestamp_id, storage_refund_amount) 
-- VALUES (1520043, 1, '0x6186f0782b5d8cc2c1d7def221675764c28a0c95dbc89d0e756c9c0eb309a864', '0x2b955cc4a0c4b3a84a0be03ec2576989b4a11dda88b303c6c43e610fcfd9f3d6', '0x1::aptos_coin::AptosCoin', NULL, 0, NULL, NULL, '0x1::coin::WithdrawEvent', False, NULL, True, '0x1::aptos_account::transfer', 757700, 'v1', '2024-10-01 00:48:33', '2024-10-01 00:48:33', '0xaa8cab62dc327ae3a215c4a68e502f5f9e1b1a1aee1d01eb5ea0fd2c515e45f6', '0x6186f0782b5d8cc2c1d7def221675764c28a0c95dbc89d0e756c9c0eb309a864', '["0x2f7b0a47d85f05e513002339d8c626f1769b9f8c1cd98cf9ec7bbf08c128f4cf", "0"]', 1727780000000.0, 0);

-- INSERT INTO public.fungible_asset_activities (transaction_version, event_index, owner_address, storage_id, asset_type, is_frozen, amount, number_used_gas_units, max_gas_price, type, is_gas_fee, gas_fee_payer_address, is_transaction_success, entry_function_id_str, block_height, token_standard, transaction_timestamp, inserted_at, txn_hash, sender, txn_args, txn_timestamp_id, storage_refund_amount) 
-- VALUES (1520043, 2, '0x2f7b0a47d85f05e513002339d8c626f1769b9f8c1cd98cf9ec7bbf08c128f4cf', '0xe48a54a43920f914fcd0fa8a306a52e34176428c366bf16bb5d3de5efeabfb5e', '0x1::aptos_coin::AptosCoin', NULL, 0, NULL, NULL, '0x1::coin::DepositEvent', False, NULL, True, '0x1::aptos_account::transfer', 757700, 'v1', '2024-10-01 00:48:33', '2024-10-01 00:48:33', '0xaa8cab62dc327ae3a215c4a68e502f5f9e1b1a1aee1d01eb5ea0fd2c515e45f6', '0x6186f0782b5d8cc2c1d7def221675764c28a0c95dbc89d0e756c9c0eb309a864', '["0x2f7b0a47d85f05e513002339d8c626f1769b9f8c1cd98cf9ec7bbf08c128f4cf", "0"]', 1727780000000.0, 0);


-- INSERT INTO public.processor_status (processor, last_success_version, last_updated, last_transaction_timestamp) 
-- VALUES ('fungible_asset_processor_nano', 2770999, '2024-10-01 10:50:07.952', '2022-10-19 07:01:40.438');

-- INSERT INTO public.processor_status (processor, last_success_version, last_updated, last_transaction_timestamp) 
-- VALUES ('user_transaction_processor', 1159987999, '2024-10-01 11:24:31.247', '2024-08-14 04:27:23.993');

-- INSERT INTO public.processor_status (processor, last_success_version, last_updated, last_transaction_timestamp) 
-- VALUES ('fungible_asset_processor_old', 2997999, '2024-09-30 18:04:44.350', '2022-10-19 08:57:23.360');

-- INSERT INTO public.processor_status (processor, last_success_version, last_updated, last_transaction_timestamp) 
-- VALUES ('fungible_asset_processor_old_micro', 2961999, '2024-10-01 09:57:01.854', '2022-10-19 08:42:15.171');

-- INSERT INTO public.processor_status (processor, last_success_version, last_updated, last_transaction_timestamp) 
-- VALUES ('fungible_asset_processor', 209852397, '2024-10-02 08:29:21.953', '2023-08-05 01:01:07.711');



-- COPY public.fungible_asset_activities (
--     transaction_version,
--     event_index,
--     owner_address,
--     storage_id,
--     asset_type,
--     is_frozen,
--     amount,
--     number_used_gas_units,
--     max_gas_price,
--     type,
--     is_gas_fee,
--     gas_fee_payer_address,
--     is_transaction_success,
--     entry_function_id_str,
--     block_height,
--     token_standard,
--     transaction_timestamp,
--     inserted_at,
--     txn_hash,
--     sender,
--     txn_args,
--     txn_timestamp_id,
--     storage_refund_amount
-- )
-- FROM '/csv-data/fungible_table.csv'
-- DELIMITER ','
-- CSV HEADER;


-- COPY public.processor_status (
--     processor,
--     last_success_version,
--     last_updated,
--     last_transaction_timestamp
-- )
-- FROM '/csv-data/processor_table.csv' 
-- DELIMITER ','
-- CSV HEADER;
