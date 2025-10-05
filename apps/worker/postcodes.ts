import { db } from "@repo/database/src";
import { and, eq, isNotNull, isNull, sql } from "@repo/database/src/orm";
import { postcodesTable, premisesTable } from "@repo/database/src/schema";
import { getNearbyPostcodes } from "./lib/api/postcodes.io/nearby";
import logger from "./logger";

export const fetchPostcodeData = async () => {
  const [row] = await db
    .select({
      postcode: premisesTable.addressPostcode,
    })
    .from(premisesTable)
    .leftJoin(
      postcodesTable,
      eq(premisesTable.addressPostcode, postcodesTable.id)
    )
    .where(
      and(isNotNull(premisesTable.addressPostcode), isNull(postcodesTable.id))
    )
    .orderBy(sql`RANDOM()`)
    .limit(1);

  if (!row || !row.postcode) {
    logger.info("No postcodes to fetch");
    return;
  }

  logger.info({ postcode: row.postcode }, "Fetching postcode data");

  const nearby = await getNearbyPostcodes(row.postcode);

  await db
    .insert(postcodesTable)
    .values(
      nearby.map(({ postcode, latitude, longitude }) => ({
        id: postcode,
        latitude,
        longitude,
      }))
    )
    .onConflictDoUpdate({
      target: postcodesTable.id,
      set: {
        latitude: sql.raw(`excluded.${postcodesTable.latitude.name}`),
        longitude: sql.raw(`excluded.${postcodesTable.longitude.name}`),
      },
    });
};
