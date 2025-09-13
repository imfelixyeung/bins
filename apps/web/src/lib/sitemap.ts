import { db } from "@repo/database/src";
import { asc } from "@repo/database/src/orm";
import { premisesTable } from "@repo/database/src/schema";

const PAGE_SIZE = 1000;

export const shouldSkipSitemapGeneration = () => {
  return !!process.env.SKIP_SITEMAP;
};

export const getPremisesSitemapPages = async () => {
  if (shouldSkipSitemapGeneration()) {
    console.info("Skipping generatiton...");
    return [];
  }

  const count = await db.$count(premisesTable);
  const pages = Math.ceil(count / PAGE_SIZE);
  return Array.from({ length: pages }).map((_, id) => ({ id }));
};

export const getPremisesSitemapPage = async (page: number) => {
  if (shouldSkipSitemapGeneration()) {
    console.info("Skipping generatiton...");
    return [];
  }

  const premises = await db
    .select({ id: premisesTable.id, updatedAt: premisesTable.updatedAt })
    .from(premisesTable)
    .orderBy(asc(premisesTable.id))
    .limit(PAGE_SIZE)
    .offset(page * PAGE_SIZE);

  return premises;
};
