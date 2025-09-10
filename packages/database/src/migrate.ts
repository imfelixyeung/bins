import { migrate } from "drizzle-orm/postgres-js/migrator";
import { db, queryClient } from ".";

(async () => {
  await migrate(db, { migrationsFolder: "./.drizzle" });
  await queryClient.end();
  console.log("Migrations complete!");
})();
