import {
  type NextFetchEvent,
  type NextRequest,
  NextResponse,
} from "next/server";
import { Ratelimit } from "@upstash/ratelimit";
import { Redis } from "@upstash/redis";

const ratelimit = new Ratelimit({
  redis: Redis.fromEnv(),
  limiter: Ratelimit.fixedWindow(10, "10s"),
  ephemeralCache: new Map(),
  prefix: "@upstash/ratelimit",
  analytics: true,
});

const rateLimitedPaths = ["/api/jobs", "/api/premises"];

export const middleware = async (
  request: NextRequest,
  context: NextFetchEvent
) => {
  const path = request.nextUrl.pathname;
  if (!rateLimitedPaths.includes(path)) {
    return NextResponse.next();
  }

  const ip = request.ip ?? request.headers.get("X-Forwarded-For") ?? "no-ip";

  const ratelimitResult = await ratelimit.limit(ip).catch((error) => {
    return error instanceof Error ? error : new Error(error);
  });

  if (ratelimitResult instanceof Error) {
    console.error(ratelimitResult);
    return NextResponse.next();
  }

  const { success, pending, limit, remaining, reset } = ratelimitResult;

  context.waitUntil(pending);

  if (!success) {
    return NextResponse.json(
      {
        success: false,
        timestamp: new Date().toISOString(),
        error: "Rate limit exceeded",
        ratelimit: { success, limit, remaining, reset },
      },
      {
        status: 429,
        headers: {
          "X-RateLimit-Success": success.toString(),
          "X-RateLimit-Limit": limit.toString(),
          "X-RateLimit-Remaining": remaining.toString(),
          "X-RateLimit-Reset": reset.toString(),
        },
      }
    );
  }

  const response = NextResponse.next();
  response.headers.set("X-RateLimit-Success", success.toString());
  response.headers.set("X-RateLimit-Limit", limit.toString());
  response.headers.set("X-RateLimit-Remaining", remaining.toString());
  response.headers.set("X-RateLimit-Reset", reset.toString());

  return response;
};

export const config = {
  matcher: "/api/:path*",
};
