CREATE TABLE dm_jobs (
    id serial PRIMARY KEY,
    premises_id integer NOT NULL,
    bin text NOT NULL,
    date date NOT NULL
);

CREATE INDEX dm_jobs_premises_id_date_idx ON dm_jobs (premises_id, date);