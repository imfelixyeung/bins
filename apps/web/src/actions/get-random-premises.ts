"use server";

import { db } from "@repo/database/src";
import { sql } from "@repo/database/src/orm";
import { unstable_cache } from "next/cache";
import { actionClient } from ".";

export const getRandomPremisesInternal = async () => {
  const now = performance.now();
  const result = await db.query.premisesTable.findFirst({
    orderBy: sql`random()`,
    columns: {
      id: true,
      addressPostcode: true,
    },
  });

  console.log("getRandomPremisesInternal", performance.now() - now);

  return result;
};

export const getCachedRandomPremises = unstable_cache(
  getRandomPremisesInternal,
  [],
  {
    revalidate: 1,
  }
);

export const getRandomPremises = actionClient.action(async () => {
  return await getCachedRandomPremises();
});
