"use server";

import { z } from "zod";
import { actionClient } from ".";
import { searchPremises } from "@/functions/search-premises";

export const searchPremisesAction = actionClient
  .inputSchema(z.object({ postcode: z.string() }))
  .action(async ({ parsedInput: { postcode } }) => {
    return await searchPremises({ postcode });
  });
