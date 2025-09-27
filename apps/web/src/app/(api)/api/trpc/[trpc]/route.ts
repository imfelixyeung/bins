import { appRouter } from "@/trpc/server/router";
import { fetchRequestHandler } from "@trpc/server/adapters/fetch";

const handler = (req: Request) => {
  return fetchRequestHandler({
    endpoint: "/api/trpc",
    req,
    router: appRouter,
  });
};

export { handler as GET, handler as POST };
