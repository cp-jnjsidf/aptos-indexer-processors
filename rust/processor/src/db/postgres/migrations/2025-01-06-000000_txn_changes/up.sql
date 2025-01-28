-- Table: public.change_resources

-- DROP TABLE IF EXISTS public.change_resources;

CREATE TABLE IF NOT EXISTS public.change_resources_partition (
    transaction_version BIGINT NOT NULL,
    transaction_block_height BIGINT NOT NULL,
    change_index BIGINT NOT NULL,
    transaction_hash VARCHAR(66) NOT NULL,
    transaction_timestamp TIMESTAMP NOT NULL,
    transaction_sender VARCHAR(66),
    transaction_entry_function_id_str text,
    is_transaction_success BOOLEAN NOT NULL,
    state_key_hash VARCHAR(66) NOT NULL,
    is_delete BOOLEAN NOT NULL,
    address VARCHAR(66) NOT NULL,
    resource_type VARCHAR NOT NULL,
    data JSONB,
    prev_transaction_version BIGINT,
    prev_change_index BIGINT,
    prev_is_delete BOOLEAN,
    prev_data JSONB,
    next_transaction_version BIGINT,
    next_change_index BIGINT,
    next_is_delete BOOLEAN,
    next_data JSONB,
    inserted_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    runner_id INT NOT NULL DEFAULT 0,
    CONSTRAINT change_resources_partition_pkey PRIMARY KEY (runner_id, transaction_version, change_index),
    CONSTRAINT data_null_when_deleted CHECK (
        ((is_delete = TRUE AND data IS NULL) OR
        (is_delete = FALSE AND data IS NOT NULL))
    )
) PARTITION BY LIST (runner_id);

DO $$
BEGIN
    FOR i IN 0..13 LOOP
        EXECUTE format('
            CREATE TABLE public.change_resources_partition_%s
            PARTITION OF public.change_resources_partition
            FOR VALUES IN (%s);
        ', i, i);
    END LOOP;
END $$;



CREATE TABLE IF NOT EXISTS public.change_table_items_partition (
    transaction_version BIGINT NOT NULL,
    transaction_block_height BIGINT NOT NULL,
    change_index BIGINT NOT NULL,
    transaction_hash VARCHAR(66) NOT NULL,
    transaction_timestamp TIMESTAMP NOT NULL,
    transaction_sender VARCHAR(66),
    transaction_entry_function_id_str text,
    is_transaction_success BOOLEAN NOT NULL,
    state_key_hash VARCHAR(66) NOT NULL,
    is_delete BOOLEAN NOT NULL,
    table_handle VARCHAR(66) NOT NULL,
    key_type text NOT NULL,
    key JSONB NOT NULL,
    value_type text,
    value JSONB,
    prev_transaction_version BIGINT,
    prev_change_index BIGINT,
    prev_is_delete BOOLEAN,
    prev_value JSONB,
    next_transaction_version BIGINT,
    next_change_index BIGINT,
    next_is_delete BOOLEAN,
    next_value JSONB,
    inserted_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    runner_id INT NOT NULL DEFAULT 0,
    CONSTRAINT change_table_items_partition_pkey PRIMARY KEY (runner_id, transaction_version, change_index),
    CONSTRAINT data_null_when_deleted_table CHECK (
        ((is_delete = TRUE AND value IS NULL) OR
        (is_delete = FALSE AND value IS NOT NULL)) AND
        ((is_delete = TRUE AND value_type IS NULL) OR
        (is_delete = FALSE AND value_type IS NOT NULL))
    )
) PARTITION BY LIST (runner_id);

DO $$
BEGIN
    FOR i IN 0..13 LOOP
        EXECUTE format('
            CREATE TABLE public.change_table_items_partition_%s
            PARTITION OF public.change_table_items_partition
            FOR VALUES IN (%s);
        ', i, i);
    END LOOP;
END $$;


CREATE TABLE IF NOT EXISTS public.change_modules (
    transaction_version BIGINT NOT NULL,
    transaction_block_height BIGINT NOT NULL,
    change_index BIGINT NOT NULL,
    transaction_hash VARCHAR(66) NOT NULL,
    transaction_timestamp TIMESTAMP NOT NULL,
    transaction_sender VARCHAR(66),
    transaction_entry_function_id_str text,
    is_transaction_success BOOLEAN NOT NULL,
    state_key_hash VARCHAR(66) NOT NULL,
    is_delete BOOLEAN NOT NULL,
    address VARCHAR(66) NOT NULL,
    name text NOT NULL,
    bytecode text,
    abi JSONB,
    package_manifest text,
    source_code text,
    is_source_correct BOOLEAN,
    inserted_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT change_modules_pkey PRIMARY KEY (transaction_version, change_index),
    CONSTRAINT data_null_when_deleted_modules CHECK (
        ((is_delete = TRUE AND bytecode IS NULL) OR
        (is_delete = FALSE AND bytecode IS NOT NULL)) AND
        ((is_delete = TRUE AND abi IS NULL) OR
        (is_delete = FALSE AND abi IS NOT NULL))
    )
);

-- Index for ascending sort by transaction_version
CREATE INDEX IF NOT EXISTS idx_resource_changes_address_txn_asc
ON public.change_resources (address, transaction_version ASC);

-- Index for descending sort by transaction_version
CREATE INDEX IF NOT EXISTS idx_resource_changes_address_txn_desc
ON public.change_resources (address, transaction_version DESC);

-- Index for ascending sort by transaction_version
CREATE INDEX IF NOT EXISTS idx_table_item_changes_table_handle_txn_asc
ON public.change_table_items (table_handle, transaction_version ASC);

-- Index for descending sort by transaction_version
CREATE INDEX IF NOT EXISTS idx_table_item_changes_table_handle_txn_desc
ON public.change_table_items (table_handle, transaction_version DESC);

CREATE INDEX IF NOT EXISTS idx_table_handle_timestamp
    ON public.change_table_items (table_handle ASC, transaction_timestamp DESC);

CREATE INDEX IF NOT EXISTS idx_transaction_address_resource
ON public.change_resources (
    transaction_version, 
    address, 
    resource_type
);

-- Index for ascending sort by transaction_version
CREATE INDEX IF NOT EXISTS idx_modules_changes_address_txn_asc
ON public.change_modules (address, transaction_version ASC);

-- Index for descending sort by transaction_version
CREATE INDEX IF NOT EXISTS idx_modules_changes_address_txn_desc
ON public.change_modules (address, transaction_version DESC);

CREATE INDEX IF NOT EXISTS idx_sender_function_version
    ON public.change_table_items (transaction_sender, transaction_entry_function_id_str, transaction_version);

CREATE INDEX IF NOT EXISTS idx_table_item_changes_composite
    ON public.change_table_items (table_handle, key, transaction_version DESC);

CREATE INDEX IF NOT EXISTS idx_sender_function_version_resources
    ON public.change_resources (transaction_sender, transaction_entry_function_id_str, transaction_version);

CREATE INDEX IF NOT EXISTS idx_address_resource_version
    ON public.change_resources (address, resource_type, transaction_version DESC);

