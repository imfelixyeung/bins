"use client";

import { searchPremisesAction } from "@/actions/search-premises-action";
import { useQuery } from "@tanstack/react-query";

export const useSearchPremisesQuery = ({
  postcode,
}: {
  postcode: string | null;
}) => {
  const query = useQuery({
    queryKey: ["search-premises", postcode],
    queryFn: () => searchPremisesAction({ postcode: postcode! }),
    enabled: !!postcode,
  });

  return query;
};
