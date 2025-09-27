"use client";

import { useTRPC } from "@/trpc/utils";
import { useQuery } from "@tanstack/react-query";

export const useSearchPremisesQuery = ({
  postcode,
}: {
  postcode: string | null;
}) => {
  const trpc = useTRPC();
  const query = useQuery(
    trpc.premises.search.queryOptions(
      { postcode: postcode! },
      { enabled: !!postcode }
    )
  );

  return query;
};
