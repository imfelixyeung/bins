-- The size of the file upstream publishes, as sent by its `content-length`
-- header. The dataset page shows this against the size of the local mirror, so a
-- dataset is only stored once upstream has answered a `HEAD` request for it.
--
-- Null for every file checked before this column existed, and for any check that
-- answered without a `content-length`.
ALTER TABLE etags
    ADD COLUMN size bigint;
