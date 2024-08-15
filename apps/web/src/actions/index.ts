import { createSafeActionClient } from "next-safe-action";

export const actionClient: ReturnType<typeof createSafeActionClient> =
  createSafeActionClient();
