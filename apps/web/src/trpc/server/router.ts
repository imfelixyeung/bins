import { publicProcedure, router } from ".";
import z from "zod";
import { searchPremises } from "@/functions/search-premises";
import { getRandomPremises } from "@/functions/get-random-premises";
import { getNearbyPostcodes } from "@/lib/api/postcodes.io/nearby";
import { getPostcodeJobs } from "@/functions/get-postcode-jobs";

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
        const nearby = await getNearbyPostcodes(input.postcode).catch(
          () => null
        );
        if (!nearby || nearby.length === 0) return null;

        const jobs = await getPostcodeJobs(nearby.map((p) => p.postcode));

        return nearby.map((postcode) => ({
          ...postcode,
          jobs: jobs.filter((job) => job.postcode === postcode.postcode),
        }));
      }),
  },
});

export type AppRouter = typeof appRouter;
