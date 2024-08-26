-- update date format to match source data
SET DateStyle TO 'ISO', "DMY";

-- drop temporary tables just in case
DROP TABLE IF EXISTS staging_dm_jobs;

-- create temporary tables to hold the data
CREATE TEMP TABLE staging_dm_jobs AS TABLE dm_jobs WITH NO DATA;

-- copy data from csv files to temporary tables
\copy staging_dm_jobs (premises_id, bin, date) FROM './temp/jobs/dm_jobs.csv' delimiter ',' CSV NULL AS '';

-- only replace all data from staging if staging contains data
BEGIN TRANSACTION;
DO $$
BEGIN
  IF (SELECT COUNT(*) FROM staging_dm_jobs) > 0 THEN
    
    -- delete all rows from dm_jobs
    DELETE FROM dm_jobs;

    -- copy data from staging
    INSERT INTO dm_jobs (premises_id, bin, date) SELECT premises_id, bin, date FROM staging_dm_jobs;

  END IF;
END $$;
COMMIT TRANSACTION;
