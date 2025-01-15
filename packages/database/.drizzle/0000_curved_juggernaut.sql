CREATE TABLE `etags` (
	`id` integer PRIMARY KEY NOT NULL,
	`url` text NOT NULL,
	`etag` text,
	`created_at` text NOT NULL,
	`updated_at` integer
);
--> statement-breakpoint
CREATE UNIQUE INDEX `etags_url_unique` ON `etags` (`url`);--> statement-breakpoint
CREATE TABLE `dm_jobs` (
	`id` integer PRIMARY KEY NOT NULL,
	`premises_id` integer NOT NULL,
	`bin` text NOT NULL,
	`date` text NOT NULL
);
--> statement-breakpoint
CREATE INDEX `jobs_premises_id_index` ON `dm_jobs` (`premises_id`);--> statement-breakpoint
CREATE TABLE `dm_premises` (
	`id` integer PRIMARY KEY NOT NULL,
	`address_room` text,
	`address_number` text,
	`address_street` text,
	`address_locality` text,
	`address_city` text,
	`address_postcode` text,
	`search_postcode` text,
	`created_at` text NOT NULL,
	`updated_at` integer
);
--> statement-breakpoint
CREATE INDEX `search_postcode_index` ON `dm_premises` (`search_postcode`);