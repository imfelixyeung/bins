import { drizzle } from "drizzle-orm/postgres-js";
import postgres, { Sql } from "postgres";
import { env } from "./env";
import * as schema from "./schema";

const globalQueryClient = globalThis as unknown as { queryClient: Sql };
export const queryClient =
  globalQueryClient.queryClient ??
  (globalQueryClient.queryClient = postgres(env.DATABASE_URL));

export const db = drizzle(queryClient, { schema });
export const dbConnect = async () => {
  return db;
};
