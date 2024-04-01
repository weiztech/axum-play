-- migrate:up

-- Check if the uuid-ossp extension is installed; create if not.
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE OR REPLACE FUNCTION generate_uid()
RETURNS text LANGUAGE plpgsql AS $$
DECLARE
    ts_part bigint := FLOOR(EXTRACT(EPOCH FROM CURRENT_TIMESTAMP) * 1000)::bigint;
    ts_part_hex text := TO_HEX(ts_part);
    uuid_raw uuid := uuid_generate_v4();
    uuid_part text := LEFT(uuid_raw::text, 8);
BEGIN
RETURN ts_part_hex || '-' || uuid_part;
END;
$$;


-- migrate:down
DROP FUNCTION IF EXISTS generate_uid();
