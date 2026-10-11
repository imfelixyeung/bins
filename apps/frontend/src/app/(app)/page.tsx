import { Typography } from "@heroui/react";
import { Suspense } from "react";
import PremisesSearchForm from "@/ui/premises-search-form";

const Page = () => {
  return (
    <div className="container py-16 grow flex flex-col justify-center">
      <section className="flex flex-col items-center justify-center gap-3">
        <Typography.Heading className="text-3xl md:text-5xl font-semibold text-balance text-center">
          Check Your Bin Days
        </Typography.Heading>
        <Typography.Paragraph className="text-muted text-xl text-center text-balance">
          Check when your bins are scheduled to be emptied.
        </Typography.Paragraph>
        <Suspense>
          <div className="w-full flex justify-center mt-16">
            <PremisesSearchForm />
          </div>
        </Suspense>
      </section>
    </div>
  );
};

export default Page;
