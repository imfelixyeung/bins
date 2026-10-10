"use client";

import { useWebMCP } from "use-webmcp-tool";
import z from "zod";
import { searchJobs, searchPremises } from "./v2api/client";

export const WebMCP = () => {
  useWebMCP({
    name: "search_premises_from_postcode",
    description:
      "Search premises or addresses using a postcode. The premises.id can then be used in show_premises_jobs_by_id or get_premises_permalink for their respective functions",
    inputSchema: z
      .object({
        postcode: z.string(),
      })
      .toJSONSchema(),
    annotations: {
      readOnlyHint: true,
    },
    async execute({ postcode }: { postcode: string }) {
      const results = await searchPremises({ postcode });
      return [
        results.length ? `Found ${results.length} results` : "No results found",
        "JSON:",
        JSON.stringify(results),
      ].join("\n");
    },
  });
  useWebMCP({
    name: "show_premises_jobs_by_id",
    description:
      "Retrieves a list of bin dates (household waste collection dates) for a given premises or address. This will include the date of collection and the type of bin (e.g. Black, Green etc.)",
    inputSchema: z
      .object({
        premises_id: z.number(),
      })
      .toJSONSchema(),
    annotations: {
      readOnlyHint: true,
    },
    async execute({ premises_id }: { premises_id: number }) {
      const result = await searchJobs({ premisesId: premises_id });
      return [
        result.jobs.length
          ? `Found ${result.jobs.length} jobs`
          : "No jobs found",
        "JSON:",
        JSON.stringify(result),
      ].join("\n");
    },
  });
  return null;
};
