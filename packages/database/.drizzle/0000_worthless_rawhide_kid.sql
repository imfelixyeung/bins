CREATE TABLE IF NOT EXISTS "dm_jobs" (
	"id" serial PRIMARY KEY NOT NULL,
	"premises_id" integer NOT NULL,
	"bin" text NOT NULL,
	"date" date NOT NULL
);
--> statement-breakpoint
CREATE TABLE IF NOT EXISTS "dm_premises" (
	"id" integer PRIMARY KEY NOT NULL,
	"address_room" text,
	"address_number" text,
	"address_street" text,
	"address_locality" text,
	"address_city" text,
	"address_postcode" text,
	"created_at" timestamp with time zone DEFAULT now() NOT NULL,
	"updated_at" timestamp with time zone DEFAULT now() NOT NULL
);
