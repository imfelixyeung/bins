import { db } from "@repo/database/src";
import { eq } from "@repo/database/src/orm";
import { premisesTable } from "@repo/database/src/schema";
import { unstable_cache } from "next/cache";

const parseIntOrZero = (value: string | null | undefined) => {
  if (!value) return 0;

  try {
    return parseInt(value);
  } catch (error) {
    return 0;
  }
};

export const searchPremises = unstable_cache(
  async ({ postcode }: { postcode: string }) => {
    const searchPostcode = postcode.toUpperCase().replace(/ /g, "");

    const result = await db.query.premisesTable.findMany({
      where: eq(premisesTable.searchPostcode, searchPostcode),
      orderBy: [
        premisesTable.addressRoom,
        premisesTable.addressNumber,
        premisesTable.addressStreet,
        premisesTable.id,
      ],
      columns: {
        id: true,
        addressRoom: true,
        addressNumber: true,
        addressStreet: true,
        addressLocality: true,
        addressCity: true,
        addressPostcode: true,
        updatedAt: true,
      },
    });

    // attempts sorting by addressNumber if possible
    // limited to 100 results to avoid performance issues
    if (result.length > 100) return result;

    try {
      const resultWithParsedInts = result.map((premise) => ({
        ...premise,
        sorter: parseIntOrZero(premise.addressNumber),
      }));
      resultWithParsedInts.sort((a, b) => a.sorter - b.sorter);

      // eslint-disable-next-line no-unused-vars
      return resultWithParsedInts.map(({ sorter: _, ...premise }) => ({
        ...premise,
      }));
    } catch (error) {
      console.error("Unable to sort by addressNumber", error);
    }

    return result;
  },
  ["searchPremises"],
  { revalidate: 60 }
);
