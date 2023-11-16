CREATE TABLE IF NOT EXISTS "collections" (
	"id" bigserial PRIMARY KEY NOT NULL,
	"premise_id" text NOT NULL,
	"bin_colour" text NOT NULL,
	"date" date NOT NULL
);
--> statement-breakpoint
CREATE TABLE IF NOT EXISTS "premises" (
	"id" text PRIMARY KEY NOT NULL,
	"address_room" text,
	"address_house_number" text,
	"address_street" text,
	"address_locality" text,
	"address_city" text,
	"address_post_code" text NOT NULL
);
