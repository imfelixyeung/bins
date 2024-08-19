import { NextRequest } from "next/server";
import { z } from "zod";
import { searchJobs } from "../../../../functions/search-jobs";
import { createIcal } from "./ical";
import { createCSV } from "./csv";

const querySchema = z.object({
  premises: z.coerce.number(),
  format: z.enum(["json", "csv", "ical"]).default("json"),
});

export const GET: any = async (request: NextRequest) => {
  const queryResult = querySchema.safeParse(
    Object.fromEntries(request.nextUrl.searchParams)
  );

  if (!queryResult.success) {
    return new Response(
      JSON.stringify({ error: true, issues: queryResult.error.issues }),
      { status: 400 }
    );
  }

  const { premises, format } = queryResult.data;

  const jobs = await searchJobs({ premisesId: premises });

  if (!jobs) {
    return new Response(
      JSON.stringify({ error: true, message: "Premises not found" }),
      { status: 404 }
    );
  }

  if (format === "ical") {
    const ical = createIcal({ data: jobs });

    return new Response(ical, {
      status: 200,
      headers: {
        "Content-Type": "text/calendar",
        "Content-Disposition": `attachment; filename="jobs-${jobs.id}.ics"`,
      },
    });
  }

  if (format === "csv") {
    const csv = createCSV({ data: jobs });

    return new Response(csv, {
      status: 200,
      headers: {
        "Content-Type": "text/csv",
        "Content-Disposition": `attachment; filename="jobs-${jobs.id}.csv"`,
      },
    });
  }

  return new Response(
    JSON.stringify({
      success: true,
      timestamp: new Date().toISOString(),
      data: jobs,
    }),
    { status: 200 }
  );
};
