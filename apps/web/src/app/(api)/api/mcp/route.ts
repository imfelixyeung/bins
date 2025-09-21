import {
  getFullAddress,
  getOneLineFullAddress,
} from "@/functions/format-address";
import { searchJobs } from "@/functions/search-jobs";
import { searchPremises } from "@/functions/search-premises";
import { format } from "date-fns";
import { createMcpHandler } from "mcp-handler";
import { z } from "zod";

const handler = createMcpHandler(
  (server) => {
    server.tool(
      "search_premises_from_postcode",
      "Search premises or addresses using a postcode. The premises.id can then be used in show_premises_jobs_by_id or get_premises_permalink for their respective functions",
      {
        postcode: z
          .string()
          .describe("The UK postcode of the address. For example, LS6 2SE"),
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
      "Retrives a list of bin dates (household waste collection dates) for a given premises or address. This will include the date of collection and the type of bin (e.g. Black, Green etc.)",
      {
        premisesId: z
          .number()
          .int()
          .nonnegative()
          .describe(
            "The premises.id returned from search_premises_from_postcode"
          ),
      },
      async ({ premisesId }) => {
        const now = format(new Date(), "yyyy-MM-dd");
        const jobs = await searchJobs({ premisesId: premisesId }).then((res) =>
          res
            ? {
                ...res,
                jobs: res.jobs.map((job) => {
                  const status =
                    job.date < now
                      ? "Expired"
                      : job.date > now
                        ? "Upcoming"
                        : "Today";
                  return { ...job, status };
                }),
              }
            : null
        );

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
                  text: `${job.date} (${job.status}): ${job.bin} bin`,
                }) as const
            ),
          ],
          structuredContent: jobs,
        };
      }
    );
    server.tool(
      "get_premises_permalink",
      "Gets the permanent link to a page for this premises or address. The page shows the full address, a simple calendar view for the next few week's collection dates, as well as the full list of bin collection dates by each bin type. The page also includes a iCal link and instructions for users to add to their preferred calendar as an integration.",
      {
        premisesId: z
          .number()
          .int()
          .nonnegative()
          .describe(
            "The premises.id returned from search_premises_from_postcode"
          ),
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

        const link = `https://bins.felixyeung.com/premises/${premisesId}`;

        return {
          content: [
            { type: "text", text: `Permalink to the premises page:\n${link}` },
          ],
          structuredContent: { link },
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
