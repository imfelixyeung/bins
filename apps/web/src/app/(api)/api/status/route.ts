import { getWorkerStatus } from "../../../../functions/get-worker-status";

export const GET: any = async () => {
  try {
    const status = await getWorkerStatus();

    return new Response(
      JSON.stringify({
        success: true,
        timestamp: new Date().toISOString(),
        data: status,
      }),
      { status: 200 }
    );
  } catch (error) {
    return new Response(
      JSON.stringify({
        success: false,
        timestamp: new Date().toISOString(),
        error: error instanceof Error ? error.message : error,
      }),
      { status: 500 }
    );
  }
};

export const dynamic = "force-dynamic";
