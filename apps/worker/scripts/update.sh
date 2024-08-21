set -e

# make temp directory
mkdir -p temp

# fetch data
wget "https://opendata.leeds.gov.uk/downloads/bins/dm_jobs.csv" -O temp/dm_jobs.csv
wget "https://opendata.leeds.gov.uk/downloads/bins/dm_premises.csv" -O temp/dm_premises.csv

# cp ../../temp/dm_jobs.csv temp/dm_jobs.csv
# cp ../../temp/dm_premises.csv temp/dm_premises.csv

# replace nulls with empty string
sed -i 's/\x00//g' temp/dm_premises.csv
sed -i 's/\x00//g' temp/dm_jobs.csv

echo "Nulls replaced!"

# run the update.sql script with psql
psql $DATABASE_URL -f scripts/update.sql

# remove the temp files
rm -rf temp

echo "Data updated!"
