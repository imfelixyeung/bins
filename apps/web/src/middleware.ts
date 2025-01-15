import {
  type NextFetchEvent,
  type NextRequest,
  NextResponse,
} from "next/server";
import {
  getRatelimit,
  getRatelimitHeaders,
  getRatelimitErrorResponse,
} from "./lib/ratelimit";

const rateLimitedPaths = ["/api/jobs", "/api/premises"];

export const middleware = async (
  request: NextRequest,
  context: NextFetchEvent
) => {
  const path = request.nextUrl.pathname;
  if (!rateLimitedPaths.includes(path)) {
    return NextResponse.next();
  }

  const ip = request.headers.get("X-Forwarded-For") ?? "no-ip";
  console.log({ ip });

  const result = await getRatelimit(request);

  const { success, pending } = result;

  context.waitUntil(pending);

  if (!success) {
    const response = await getRatelimitErrorResponse(result);
    return response;
  }

  const response = NextResponse.next();
  const headers = await getRatelimitHeaders(result);
  for (const [key, value] of Object.entries(headers)) {
    response.headers.set(key, value);
  }
  response.headers.set("Cache-Control", "public, max-age=3600");

  return response;
};

export const config = {
  matcher: "/api/:path*",
};
