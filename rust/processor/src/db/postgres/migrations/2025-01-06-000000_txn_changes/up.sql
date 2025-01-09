-- Table: public.change_resources

-- DROP TABLE IF EXISTS public.change_resources;

CREATE TABLE IF NOT EXISTS public.change_resources (
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
    inserted_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT change_resources_pkey PRIMARY KEY (transaction_version, change_index),
    CONSTRAINT data_null_when_deleted CHECK (
        ((is_delete = TRUE AND data IS NULL) OR
        (is_delete = FALSE AND data IS NOT NULL))
    )
);



CREATE TABLE IF NOT EXISTS public.change_table_items (
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
    inserted_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT change_table_items_pkey PRIMARY KEY (transaction_version, change_index),
    CONSTRAINT data_null_when_deleted_table CHECK (
        ((is_delete = TRUE AND value IS NULL) OR
        (is_delete = FALSE AND value IS NOT NULL))
    )
);

TABLESPACE pg_default;

ALTER TABLE IF EXISTS public.change_resources
    OWNER to postgres;

ALTER TABLE IF EXISTS public.change_table_items
    OWNER to postgres;

-- Index for ascending sort by transaction_version
CREATE INDEX IF NOT EXISTS idx_resource_changes_address_txn_asc
ON public.change_resources (address, transaction_version ASC);

-- Index for descending sort by transaction_version
CREATE INDEX IF NOT EXISTS idx_resource_changes_address_txn_desc
ON public.change_resources (address, transaction_version DESC);

-- Index for ascending sort by transaction_version
CREATE INDEX IF NOT EXISTS idx_table_item_changes_address_txn_asc
ON public.change_table_items (address, transaction_version ASC);

-- Index for descending sort by transaction_version
CREATE INDEX IF NOT EXISTS idx_table_item_changes_address_txn_desc
ON public.change_table_items (address, transaction_version DESC);