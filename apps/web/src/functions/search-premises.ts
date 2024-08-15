import { db } from "@repo/database/src";
import { eq } from "@repo/database/src/orm";
import { premisesTable } from "@repo/database/src/schema";
import { unstable_cache } from "next/cache";

export const searchPremises = unstable_cache(
  async ({ postcode }: { postcode: string }) => {
    const result = await db.query.premisesTable.findMany({
      where: eq(premisesTable.searchPostcode, postcode),
      columns: {
        createdAt: false,
      },
    });

    return result;
  },
  ["searchPremises"],
  { revalidate: 60 }
);
