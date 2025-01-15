CREATE TABLE dm_premises(
  id INTEGER PRIMARY KEY,
  address_1 TEXT,
  address_2 TEXT,
  address_3 TEXT,
  locality TEXT,
  city TEXT,
  postcode TEXT
);

.mode csv
.import dm_premises.csv dm_premises