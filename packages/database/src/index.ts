import { drizzle } from "drizzle-orm/libsql";
import { env } from "./env";
import * as schema from "./schema";

export const db = drizzle({
  connection: {
    url: env.DATABASE_URL,
    authToken: env.DATABASE_TOKEN,
  },
  schema,
});
export const dbConnect = async () => {
  return db;
};
