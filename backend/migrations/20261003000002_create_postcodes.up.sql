CREATE TABLE postcodes (
    id text PRIMARY KEY,
    latitude double precision NOT NULL CHECK (latitude BETWEEN -90 AND 90),
    longitude double precision NOT NULL CHECK (longitude BETWEEN -180 AND 180),
    location geography(Point, 4326)
        GENERATED ALWAYS AS (
            ST_SetSRID(ST_MakePoint(longitude, latitude), 4326)::geography
        ) STORED,
    created_at timestamp with time zone NOT NULL DEFAULT now(),
    updated_at timestamp with time zone NOT NULL DEFAULT now()
);

CREATE INDEX postcodes_location_idx ON postcodes USING gist (location);

CREATE TRIGGER postcodes_set_updated_at
    BEFORE UPDATE ON postcodes
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();