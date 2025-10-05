import { db } from "@repo/database/src";
import { and, eq, gte, lt, lte } from "@repo/database/src/orm";
import { jobsTable, premisesTable } from "@repo/database/src/schema";
import { addDays, format } from "date-fns";

export const getPostcodeMap = async () => {
  const postcodes = await db.query.postcodesTable.findMany({
    columns: { id: true, latitude: true, longitude: true },
  });

  const today = format(new Date(), "yyyy-MM-dd");
  const nextWeek = format(addDays(new Date(), 7), "yyyy-MM-dd");

  const jobs = await db
    .select({
      bin: jobsTable.bin,
      date: jobsTable.date,
      postcode: premisesTable.addressPostcode,
    })
    .from(premisesTable)
    .innerJoin(jobsTable, eq(premisesTable.id, jobsTable.premisesId))
    .where(and(gte(jobsTable.date, today), lte(jobsTable.date, nextWeek)))
    .groupBy(jobsTable.bin, jobsTable.date, premisesTable.addressPostcode)
    .orderBy(premisesTable.addressPostcode, jobsTable.date, jobsTable.bin);

  return postcodes.map(({ id, latitude, longitude }) => ({
    postcode: id,
    latitude,
    longitude,
    jobs: jobs.filter((job) => job.postcode === id),
  }));
};
