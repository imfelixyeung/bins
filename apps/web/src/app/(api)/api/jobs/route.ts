import { NextRequest } from "next/server";
import { z } from "zod";
import { searchJobs } from "../../../../functions/search-jobs";

const querySchema = z.object({
  premises: z.coerce.number(),
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

  const { premises } = queryResult.data;

  const jobs = await searchJobs({ premisesId: premises });

  if (!jobs) {
    return new Response(
      JSON.stringify({ error: true, message: "Premises not found" }),
      { status: 404 }
    );
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
