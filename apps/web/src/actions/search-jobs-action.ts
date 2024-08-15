"use server";

import { z } from "zod";
import { actionClient } from ".";
import { searchJobs } from "@/functions/search-jobs";

export const searchJobsAction = actionClient
  .schema(z.object({ premises: z.number() }))
  .action(async ({ parsedInput: { premises } }) => {
    return await searchJobs({ premisesId: premises });
  });
