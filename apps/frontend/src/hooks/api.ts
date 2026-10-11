import { useQuery } from "@tanstack/react-query";
import {
  getDatasetHistory,
  getDatasets,
  getNearbyPostcodes,
  getRandomPremises,
  searchJobs,
  searchPremises,
} from "@/v2api/client";

type QueryOption = Parameters<typeof useQuery>[0];

export const useSearchPremisesQuery = ({ postcode }: { postcode: string }) =>
  useQuery({
    queryKey: ["premise", postcode],
    queryFn: () => searchPremises({ postcode }),
    enabled: postcode.trim().length > 0,
  });

const useRandomPremisesQueryOptions = {
  queryKey: ["getRandomPremise"],
  queryFn: getRandomPremises,
  enabled: false,
} as const satisfies QueryOption;

export const useRandomPremisesQuery = () =>
  useQuery(useRandomPremisesQueryOptions);

export const makeUsePremisesJobsQueryOptions = ({
  premisesId,
}: {
  premisesId: number;
}) =>
  ({
    queryKey: ["searchJobs", premisesId],
    queryFn: () => searchJobs({ premisesId }),
  }) as const satisfies QueryOption;

export const usePremisesJobsQuery = ({ premisesId }: { premisesId: number }) =>
  useQuery(makeUsePremisesJobsQueryOptions({ premisesId }));

export const useNearbyPostcodesQuery = ({ postcode }: { postcode: string }) =>
  useQuery({
    queryKey: ["getNearbyPostcodes", postcode],
    queryFn: () => getNearbyPostcodes({ postcode }),
  });

/**
 * How often the datasets are asked for again while a sync is running, in
 * milliseconds. A sync of the collections file takes minutes, so the page
 * reports progress rather than waiting for the sync to end.
 */
const WHILE_SYNCING_INTERVAL = 5_000;

export const useDatasetsQuery = () =>
  useQuery({
    queryKey: ["getDatasets"],
    queryFn: getDatasets,
    refetchInterval: (query) =>
      query.state.data?.datasets.some(
        (dataset) => dataset.sync?.state === "running",
      )
        ? WHILE_SYNCING_INTERVAL
        : false,
  });

export const useDatasetHistoryQuery = ({ target }: { target: string }) =>
  useQuery({
    queryKey: ["getDatasetHistory", target],
    queryFn: () => getDatasetHistory({ target }),
    refetchInterval: (query) =>
      query.state.data?.[0]?.state === "running"
        ? WHILE_SYNCING_INTERVAL
        : false,
  });
