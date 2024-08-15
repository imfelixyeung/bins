source ../.env

# fetch data
wget "https://opendata.leeds.gov.uk/downloads/bins/dm_jobs.csv"
wget "https://opendata.leeds.gov.uk/downloads/bins/dm_premises.csv"

# replace nulls with empty string
sed -i 's/\x00//g' dm_premises.csv
sed -i 's/\x00//g' dm_jobs.csv

# run the update.sql script with psql
psql $DATABASE_URL -f update.sql

# remove the temp files
rm dm_jobs.csv
rm dm_premises.csv

echo "Data updated!"
