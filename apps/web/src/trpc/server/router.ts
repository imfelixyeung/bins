import { publicProcedure, router } from ".";
import z from "zod";
import { searchPremises } from "@/functions/search-premises";
import { getRandomPremises } from "@/functions/get-random-premises";

export const appRouter = router({
  greeting: publicProcedure.query(() => "hello tRPC v10!"),
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
});

export type AppRouter = typeof appRouter;
