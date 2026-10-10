import { Suspense } from "react";
import Client from "./page.client";

const Page = () => {
  return (
    <Suspense>
      <Client />
    </Suspense>
  );
};

export default Page;
