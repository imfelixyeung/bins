import { type NextRequest, NextResponse } from "next/server";

export const middleware = async (request: NextRequest) => {
  const proxySecret = request.headers.get("X-RapidAPI-Proxy-Secret");

  if (proxySecret !== process.env.RAPID_API_PROXY_SECRET) {
    return new NextResponse("Bad request", { status: 400 });
  }

  const response = NextResponse.next();
  return response;
};

export const config = {
  matcher: "/api/:path*",
};
