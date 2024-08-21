CREATE TABLE IF NOT EXISTS "dm_etags" (
	"id" serial PRIMARY KEY NOT NULL,
	"url" text NOT NULL,
	"etag" text NOT NULL,
	"created_at" timestamp with time zone DEFAULT now() NOT NULL,
	"updated_at" timestamp with time zone DEFAULT now() NOT NULL,
	CONSTRAINT "dm_etags_url_unique" UNIQUE("url")
);
