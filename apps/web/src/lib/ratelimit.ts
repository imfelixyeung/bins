import { Ratelimit } from "@upstash/ratelimit";
import { NextRequest, NextResponse } from "next/server";
import { Redis } from "@upstash/redis";

const redis = Redis.fromEnv();

const ratelimit = new Ratelimit({
  redis: redis,
  limiter: Ratelimit.fixedWindow(10, "10s"),
  ephemeralCache: new Map(),
  prefix: "@upstash/ratelimit",
  analytics: true,
});

export type RatelimitResult = Awaited<ReturnType<typeof ratelimit.limit>>;

export const getRatelimit = async (
  request: NextRequest
): Promise<RatelimitResult> => {
  // Header X-RapidAPI-Proxy-Secret
  const proxySecret = request.headers.get("X-RapidAPI-Proxy-Secret");

  if (proxySecret === process.env.RAPID_API_PROXY_SECRET) {
    return {
      success: true,
      limit: 999999,
      remaining: 999999,
      reset: Date.now(),
      pending: new Promise(() => null),
    };
  }

  const ip = request.headers.get("X-Forwarded-For") ?? "no-ip";
  const result = await ratelimit.limit(ip);

  return { ...result };
};

export const getRatelimitHeaders = async (result: RatelimitResult) => {
  const { success, limit, remaining, reset } = result;

  return {
    "X-RateLimit-Success": success.toString(),
    "X-RateLimit-Limit": limit.toString(),
    "X-RateLimit-Remaining": remaining.toString(),
    "X-RateLimit-Reset": reset.toString(),
  };
};

export const getRatelimitErrorResponse = async (result: RatelimitResult) => {
  const { success, limit, remaining, reset } = result;

  const headers = await getRatelimitHeaders(result);
  return NextResponse.json(
    {
      success: false,
      timestamp: new Date().toISOString(),
      error: "Rate limit exceeded",
      ratelimit: { success, limit, remaining, reset },
    },
    {
      status: 429,
      headers,
    }
  );
};
