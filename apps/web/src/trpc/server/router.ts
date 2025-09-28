import { publicProcedure, router } from ".";
import z from "zod";
import { searchPremises } from "@/functions/search-premises";
import { getRandomPremises } from "@/functions/get-random-premises";
import { postcodeAutocomplete } from "@/functions/postcode-autocomplete";

export const appRouter = router({
  premises: {
    search: publicProcedure
      .input(z.object({ postcode: z.string() }))
      .query(async ({ input }) => {
        return await searchPremises(input);
      }),

    random: publicProcedure.query(async () => {
      return await getRandomPremises();
    }),
  },
  postcode: {
    autocomplete: publicProcedure
      .input(z.object({ postcode: z.string() }))
      .query(async ({ input }) => {
        return await postcodeAutocomplete(input.postcode);
      }),
  },
});

export type AppRouter = typeof appRouter;
