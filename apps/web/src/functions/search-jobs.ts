import { db } from "@repo/database/src";
import { eq } from "@repo/database/src/orm";
import { premisesTable } from "@repo/database/src/schema";
import { unstable_cache } from "next/cache";

export const searchJobs = unstable_cache(
  async ({ premisesId }: { premisesId: number }) => {
    const result = await db.query.premisesTable.findFirst({
      where: eq(premisesTable.id, premisesId),
      columns: {
        createdAt: false,
      },
      with: { jobs: { columns: { id: false, premisesId: false } } },
    });

    return result;
  },
  ["searchJobs"],
  { revalidate: 60 }
);

// takes the non-null non-undefined awaited return type of searchJobs
export type ReturnedJobs = NonNullable<Awaited<ReturnType<typeof searchJobs>>>;
