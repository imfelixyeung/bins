ALTER TABLE "dm_premises" ADD COLUMN "search_postcode" text;--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "jobs_premises_id_index" ON "dm_jobs" USING btree ("premises_id");--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "search_postcode_index" ON "dm_premises" USING btree ("search_postcode");