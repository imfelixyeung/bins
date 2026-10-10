import { Suspense } from "react";
import Client from "./page.client";

/**
 * The datasets that have a history page. Only these are exported, so anything
 * else under `/datasets` is not a page at all.
 */
export const dynamicParams = false;

export const generateStaticParams = () => {
  return [{ target: "jobs" }, { target: "premises" }];
};

const Page = async ({ params }: { params: Promise<{ target: string }> }) => {
  const { target } = await params;

  return (
    <Suspense>
      <Client target={target} />
    </Suspense>
  );
};

export default Page;
