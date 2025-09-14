import {
  getFullAddress,
  getOneLineFullAddress,
} from "@/functions/format-address";
import { searchJobs } from "@/functions/search-jobs";
import { searchPremises } from "@/functions/search-premises";
import { createMcpHandler } from "mcp-handler";
import { z } from "zod";

const handler = createMcpHandler(
  (server) => {
    server.tool(
      "search_premises_from_postcode",
      "Search premises using a postcode",
      {
        postcode: z.string(),
      },
      async ({ postcode }) => {
        const premises = await searchPremises({ postcode });

        if (premises.length === 0)
          return {
            isError: true,
            content: [
              {
                type: "text",
                text: `No address matching with postcode ${postcode} found`,
              },
            ],
          };

        return {
          content: [
            ...premises.map(
              (premises) =>
                ({
                  type: "text",
                  text: `${premises.id}: ${getOneLineFullAddress(premises)}`,
                }) as const
            ),
          ],
          structuredContent: { premises },
        };
      }
    );
    server.tool(
      "show_premises_jobs_by_id",
      "Retrives a list of bin dates for a given premises",
      {
        premisesId: z.number().int().nonnegative(),
      },
      async ({ premisesId }) => {
        const jobs = await searchJobs({ premisesId: premisesId });

        if (!jobs)
          return {
            isError: true,
            content: [
              {
                type: "text",
                text: `Address not found (bad premisesId of ${premisesId})`,
              } as const,
            ],
          };

        return {
          content: [
            { type: "text", text: `Full Address:\n${getFullAddress(jobs)}` },
            ...jobs.jobs.map(
              (job) =>
                ({
                  type: "text",
                  text: `${job.date}: ${job.bin} bin`,
                }) as const
            ),
          ],
          structuredContent: jobs,
        };
      }
    );
  },
  {},
  {
    basePath: "/api",
    maxDuration: 5,
    verboseLogs: true,
  }
);

export { handler as GET, handler as POST };
