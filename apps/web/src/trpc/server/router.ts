import { publicProcedure, router } from ".";
import z from "zod";
import {
  getNearbyPostcodes,
  getRandomPremises,
  searchPremises,
} from "@/v2api/client";

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
  nearby: {
    get: publicProcedure
      .input(z.object({ postcode: z.string() }))
      .query(async ({ input }) => {
        // The backend answers with null when it has no coordinates for the
        // postcode, which is what the map draws nothing for.
        return await getNearbyPostcodes(input).catch(() => null);
      }),
  },
});

export type AppRouter = typeof appRouter;
