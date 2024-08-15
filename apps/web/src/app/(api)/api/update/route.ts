import { triggerWorkerUpdate } from "../../../../functions/trigger-worker-update";

export const POST: any = async () => {
  try {
    const result = await triggerWorkerUpdate();

    return new Response(
      JSON.stringify({
        success: true,
        timestamp: new Date().toISOString(),
        data: result,
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
