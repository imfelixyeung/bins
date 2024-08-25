-- update date format to match source data
SET DateStyle TO 'ISO', "DMY";

-- delete all rows from dm_premises
DELETE FROM dm_premises;

-- copy data from csv file to dm_premises table
\copy dm_premises (id, address_room, address_number, address_street, address_locality, address_city, address_postcode) FROM './temp_premises/dm_premises.csv' delimiter ',' CSV NULL AS '';

-- copy address_postcode column to search_postcode column, removing spaces, and converting to upper case
UPDATE dm_premises SET search_postcode = upper(replace(address_postcode, ' ', ''));
