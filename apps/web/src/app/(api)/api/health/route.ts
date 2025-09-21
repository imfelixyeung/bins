import { NextResponse } from "next/server";

export const dynamic = "force-static";

export const GET = async () =>
  NextResponse.json({ timestamp: new Date().toISOString() });
