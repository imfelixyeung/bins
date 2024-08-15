-- update date format to match source data
SET DateStyle TO 'ISO', "DMY";

-- delete all rows from dm_premises and dm_jobs
DELETE FROM dm_premises;
DELETE FROM dm_jobs;

-- copy data from csv files to dm_premises and dm_jobs tables
\copy dm_premises (id, address_room, address_number, address_street, address_locality, address_city, address_postcode) FROM '/home/felixyeung/github.com/felixyeungdev/bins/temp/dm_premises.csv' delimiter ',' CSV NULL AS '';
\copy dm_jobs (premises_id, bin, date) FROM '/home/felixyeung/github.com/felixyeungdev/bins/temp/dm_jobs.csv' delimiter ',' CSV NULL AS '';
