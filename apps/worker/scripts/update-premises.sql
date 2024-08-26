-- update date format to match source data
SET DateStyle TO 'ISO', "DMY";

-- drop temporary tables just in case
DROP TABLE IF EXISTS staging_dm_premises;

-- create temporary tables to hold the data
CREATE TEMP TABLE staging_dm_premises AS TABLE dm_premises WITH NO DATA;

-- copy data from csv files to temporary tables
\copy staging_dm_premises (id, address_room, address_number, address_street, address_locality, address_city, address_postcode) FROM './temp/premises/dm_premises.csv' delimiter ',' CSV NULL AS '';

-- copy address_postcode column to search_postcode column, removing spaces, and converting to upper case
UPDATE staging_dm_premises SET search_postcode = upper(replace(address_postcode, ' ', ''));

-- only replace all data from staging if staging contains data
BEGIN TRANSACTION;
DO $$
BEGIN
  IF (SELECT COUNT(*) FROM staging_dm_premises) > 0 THEN

    -- delete all rows from dm_premises
    DELETE FROM dm_premises;

    -- copy data from staging
    INSERT INTO dm_premises
      (id, address_room, address_number, address_street, address_locality, address_city, address_postcode, search_postcode)
    SELECT
      id, address_room, address_number, address_street, address_locality, address_city, address_postcode, search_postcode
    FROM staging_dm_premises;

  END IF;
END $$;
COMMIT TRANSACTION;
