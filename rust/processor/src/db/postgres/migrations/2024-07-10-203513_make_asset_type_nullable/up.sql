-- Your SQL goes here
ALTER TABLE IF EXISTS fungible_asset_activities ALTER COLUMN number_used_gas_units DROP NOT NULL;
ALTER TABLE IF EXISTS fungible_asset_activities ALTER COLUMN owner_address DROP NOT NULL;

 numeric,
    max_gas_price numeric,