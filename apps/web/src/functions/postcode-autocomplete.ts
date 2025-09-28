import { db } from "@repo/database/src";
import { and, isNotNull, like, sql } from "@repo/database/src/orm";
import { premisesTable } from "@repo/database/src/schema";

export const postcodeAutocomplete = async (postcode: string) => {
  const searchPostcode = postcode.toUpperCase().replace(/ /g, "");

  const results = await db
    .selectDistinctOn([premisesTable.addressPostcode], {
      postcode: premisesTable.addressPostcode,
    })
    .from(premisesTable)
    .where(
      and(
        isNotNull(premisesTable.addressPostcode),
        like(premisesTable.searchPostcode, `${searchPostcode}%`)
      )
    )
    .limit(20);

  return results.map(({ postcode }) => postcode!);
};
