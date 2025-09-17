import { db } from "@repo/database/src";
import { sql } from "@repo/database/src/orm";

export const vacuumFull = async () => {
  await db.execute(sql`VACUUM FULL`);
};
