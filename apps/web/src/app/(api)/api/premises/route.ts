import { NextRequest } from "next/server";
import { z } from "zod";
import { searchPremises } from "../../../../functions/search-premises";

const querySchema = z.object({
  postcode: z.string(),
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

  const { postcode } = queryResult.data;

  const premises = await searchPremises({ postcode });

  return new Response(
    JSON.stringify({
      success: true,
      timestamp: new Date().toISOString(),
      data: premises,
    }),
    { status: 200 }
  );
};
