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

COMMIT;
