set -e

TEMP_DIR=temp-jobs

# make temp directory
mkdir -p $TEMP_DIR

# fetch data
wget "https://opendata.leeds.gov.uk/downloads/bins/dm_jobs.csv" -O $TEMP_DIR/dm_jobs.csv

# cp ../../$TEMP_DIR/dm_jobs.csv $TEMP_DIR/dm_jobs.csv

# replace nulls with empty string
sed -i 's/\x00//g' $TEMP_DIR/dm_jobs.csv

echo "Nulls replaced!"

# run the update.sql script with psql
psql $DATABASE_URL -f scripts/update-jobs.sql

# remove the temp files
rm -rf $TEMP_DIR

echo "Data updated!"
