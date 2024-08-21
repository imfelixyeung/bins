ALTER TABLE "dm_etags" RENAME TO "etags";--> statement-breakpoint
ALTER TABLE "etags" DROP CONSTRAINT "dm_etags_url_unique";--> statement-breakpoint
ALTER TABLE "etags" ADD CONSTRAINT "etags_url_unique" UNIQUE("url");