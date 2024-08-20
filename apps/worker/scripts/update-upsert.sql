/*
 * concept for upserting instead of replacing everything
 * did not work as well, it takes even longer to run
 * archiving here for reference
 */

-- update date format to match source data
SET DateStyle TO 'ISO', "DMY";


-- drop temporary tables
DROP TABLE IF EXISTS staging_dm_premises;

DROP TABLE IF EXISTS staging_dm_jobs;


-- create temporary tables to hold the data
CREATE TEMP TABLE staging_dm_premises AS TABLE dm_premises WITH NO DATA;

CREATE TEMP TABLE staging_dm_jobs AS TABLE dm_jobs WITH NO DATA;


-- copy data from csv files to temporary tables
\copy staging_dm_premises (id, address_room, address_number, address_street, address_locality, address_city, address_postcode) FROM './temp/dm_premises.csv' DELIMITER ',' CSV NULL AS '';

\copy staging_dm_jobs (premises_id, bin, date) FROM './temp/dm_jobs.csv' delimiter ',' CSV NULL AS '';


-- delete rows that are not in the csv file
DELETE FROM dm_premises
WHERE id NOT IN (SELECT id FROM staging_dm_premises);

DELETE FROM dm_jobs
WHERE (premises_id, bin, date) NOT IN (SELECT premises_id, bin, date FROM staging_dm_jobs);


-- perform upsert operation on temporary table to dm_premises table
INSERT INTO dm_premises (id, address_room, address_number, address_street, address_locality, address_city, address_postcode)
SELECT id, address_room, address_number, address_street, address_locality, address_city, address_postcode
FROM staging_dm_premises
ON CONFLICT (id) -- Assumes 'id' is the unique key (primary key) for the dm_premises table
DO UPDATE SET
    address_room = EXCLUDED.address_room,
    address_number = EXCLUDED.address_number,
    address_street = EXCLUDED.address_street,
    address_locality = EXCLUDED.address_locality,
    address_city = EXCLUDED.address_city,
    address_postcode = EXCLUDED.address_postcode,
    updated_at = NOW()
WHERE dm_premises.address_room IS DISTINCT FROM EXCLUDED.address_room
   OR dm_premises.address_number IS DISTINCT FROM EXCLUDED.address_number
   OR dm_premises.address_street IS DISTINCT FROM EXCLUDED.address_street
   OR dm_premises.address_locality IS DISTINCT FROM EXCLUDED.address_locality
   OR dm_premises.address_city IS DISTINCT FROM EXCLUDED.address_city
   OR dm_premises.address_postcode IS DISTINCT FROM EXCLUDED.address_postcode;

INSERT INTO dm_jobs (premises_id, bin, date)
SELECT premises_id, bin, date
FROM staging_dm_jobs
ON CONFLICT (premises_id, bin, date) DO NOTHING;


-- copy address_postcode column to search_postcode column, removing spaces, and converting to upper case
UPDATE dm_premises SET search_postcode = upper(replace(address_postcode, ' ', ''));


-- drop temporary tables
DROP TABLE IF EXISTS staging_dm_premises;
DROP TABLE IF EXISTS staging_dm_jobs;
