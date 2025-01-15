wget https://opendata.leeds.gov.uk/downloads/bins/dm_jobs.csv -O dm_jobs.csv
wget https://opendata.leeds.gov.uk/downloads/bins/dm_premises.csv -O dm_premises.csv

DATABASE_FILE=dm.db

# delete if exists
rm -f $DATABASE_FILE

sqlite3 $DATABASE_FILE < create.sql