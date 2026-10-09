"use client";

import { useQuery } from "@tanstack/react-query";
import { getNearbyPostcodes } from "@/v2api/client";
import NearbyMapClient from "./nearby-map.client";

const NearbyMap = ({ postcode }: { postcode: string }) => {
  const nearby = useQuery({
    queryKey: [getNearbyPostcodes, postcode],
    queryFn: () => getNearbyPostcodes({ postcode }),
  });

  if (nearby.isLoading) {
    return (
      <div className="my-16 min-h-64 w-full grid rounded-xl bg-gray-200 animate-pulse" />
    );
  }

  if (nearby.error || !nearby.data) {
    return null;
  }

  return <NearbyMapClient nearby={nearby.data} />;
};

export default NearbyMap;
