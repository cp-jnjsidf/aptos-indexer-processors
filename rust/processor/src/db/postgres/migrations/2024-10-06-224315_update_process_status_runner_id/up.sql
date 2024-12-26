CREATE TABLE IF NOT EXISTS processor_status (LIKE processor_status_old INCLUDING ALL);

BEGIN;

-- Step 1: Add columns if they do not already exist
ALTER TABLE processor_status
ADD COLUMN IF NOT EXISTS runner_id BIGINT NOT NULL DEFAULT 0,
ADD COLUMN IF NOT EXISTS start_version BIGINT NOT NULL DEFAULT 0,
ADD COLUMN IF NOT EXISTS upper_bound BIGINT;

-- Step 2: Set any NULL values to default values (if necessary)
UPDATE processor_status
SET runner_id = 0
WHERE runner_id IS NULL;

-- Step 3: Drop existing primary key constraint, if it exists
ALTER TABLE processor_status
DROP CONSTRAINT IF EXISTS processor_status_pkey1;

-- Step 4: Add composite primary key
ALTER TABLE processor_status
ADD PRIMARY KEY (processor, runner_id);

-- Step 5: Define variables and insert data only if the table is empty
DO $$
DECLARE
    initial_start_version BIGINT := 1161084999; -- Set this value to your desired initial start version
    initial_end_version BIGINT := 1772069791;   -- Set this value to your desired end version
    -- initial_start_version BIGINT := 0; -- Set this value to your desired initial start version
    -- initial_end_version BIGINT := 10000000;   -- Set this value to your desired end version
    num_rows INTEGER := 4;                      -- Set the number of rows to insert
    version_step BIGINT;                        -- Step size to distribute versions evenly

    current_version BIGINT;
    runner_id_counter INTEGER;
BEGIN
    -- Calculate the version step size for equal distribution
    version_step := (initial_end_version - initial_start_version) / num_rows;

    -- Insert data only if there are no rows with 'fungible_asset_processor' processor
    IF (SELECT COUNT(*) FROM processor_status WHERE processor = 'fungible_asset_processor') = 0 THEN
        -- Initialize variables
        current_version := initial_start_version - 1;
        runner_id_counter := 0;

        -- Loop to insert multiple rows based on the number of rows required
        FOR i IN 1..num_rows LOOP
            -- Insert a new row with dynamically set values
            INSERT INTO processor_status (processor, last_updated, last_transaction_timestamp, runner_id, start_version, last_success_version, upper_bound)
            VALUES (
                'fungible_asset_processor', 
                '2022-01-01 00:00:00', 
                '2022-01-01 00:00:00', 
                runner_id_counter, 
                current_version, 
                current_version, 
                CASE 
                    WHEN i < num_rows THEN current_version + version_step -- Set upper_bound for all rows except the last one
                    ELSE NULL 
                END
            );

            -- Update variables for the next iteration
            current_version := current_version + version_step;
            runner_id_counter := runner_id_counter + 1;
        END LOOP;
    END IF;
END $$;

COMMIT;
