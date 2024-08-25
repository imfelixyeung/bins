-- update date format to match source data
SET DateStyle TO 'ISO', "DMY";

-- delete all rows from dm_jobs
DELETE FROM dm_jobs;

-- copy data from csv files to dm_jobs table
\copy dm_jobs (premises_id, bin, date) FROM './temp_jobs/dm_jobs.csv' delimiter ',' CSV NULL AS '';
