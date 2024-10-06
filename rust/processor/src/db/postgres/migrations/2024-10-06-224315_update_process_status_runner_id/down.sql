-- Start a transaction to ensure atomicity
BEGIN;

-- Step 1: Drop the primary key constraint that was added in `up.sql`
ALTER TABLE processor_status DROP CONSTRAINT IF EXISTS processor_status_pkey;

-- Step 2: Drop the unique constraint if it was created
ALTER TABLE processor_status DROP CONSTRAINT IF EXISTS processor_runner_unique;

-- Step 3: Drop the columns `runner_id`, `start_version`, and `upper_bound` that were added
ALTER TABLE processor_status
DROP COLUMN IF EXISTS runner_id,
DROP COLUMN IF EXISTS start_version,
DROP COLUMN IF EXISTS upper_bound;

-- Commit the transaction
COMMIT;
