import { db } from "@repo/database/src";
import { eq } from "@repo/database/src/orm";
import { jobsTable, premisesTable } from "@repo/database/src/schema";
import { unstable_cache } from "next/cache";

export const searchJobs = unstable_cache(
  async ({ premisesId }: { premisesId: number }) => {
    const result = await db.query.premisesTable.findFirst({
      where: eq(premisesTable.id, premisesId),
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
      with: {
        jobs: {
          columns: {
            date: true,
            bin: true,
          },
          orderBy: [jobsTable.date, jobsTable.bin, jobsTable.id],
        },
      },
    });

    return result;
  },
  ["searchJobs"],
  { revalidate: 60 }
);

// takes the non-null non-undefined awaited return type of searchJobs
export type ReturnedJobs = NonNullable<Awaited<ReturnType<typeof searchJobs>>>;
