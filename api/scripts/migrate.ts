import { env } from "@/env";
import { logger } from "@/logger";
import { drizzle } from "drizzle-orm/postgres-js";
import { migrate } from "drizzle-orm/postgres-js/migrator";
import postgres from "postgres";

const migrationClient = postgres(env.DB_URL, { max: 1 });

logger.info("Running migrations");
await migrate(drizzle(migrationClient), { migrationsFolder: "./drizzle" });
logger.info("Migrations complete");

process.exit(0);
