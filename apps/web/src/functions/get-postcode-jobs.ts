import { db } from "@repo/database/src";
import { eq, inArray } from "@repo/database/src/orm";
import { jobsTable, premisesTable } from "@repo/database/src/schema";

export const getPostcodeJobs = async (postcodes: string[]) => {
  // Prevent too many postcodes
  if (postcodes.length >= 150) return [];

  return await db
    .select({
      bin: jobsTable.bin,
      date: jobsTable.date,
      postcode: premisesTable.addressPostcode,
    })
    .from(premisesTable)
    .innerJoin(jobsTable, eq(premisesTable.id, jobsTable.premisesId))
    .where(inArray(premisesTable.addressPostcode, postcodes))
    .groupBy(jobsTable.bin, jobsTable.date, premisesTable.addressPostcode)
    .orderBy(premisesTable.addressPostcode, jobsTable.date, jobsTable.bin);
};
