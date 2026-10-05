CREATE TABLE dm_premises (
    id serial PRIMARY KEY,
    address_room text,
    address_number text,
    address_street text,
    address_locality text,
    address_city text,
    address_postcode text,
    search_postcode text,
    created_at timestamp with time zone NOT NULL DEFAULT now(),
    updated_at timestamp with time zone NOT NULL DEFAULT now()
);

CREATE INDEX dm_premises_search_postcode_idx ON dm_premises (search_postcode);

CREATE INDEX dm_premises_address_postcode_idx ON dm_premises (address_postcode);

CREATE TRIGGER dm_premises_set_updated_at
    BEFORE UPDATE ON dm_premises
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();