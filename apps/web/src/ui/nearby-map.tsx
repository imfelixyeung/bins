"use client";

import React from "react";
import NearbyMapClient from "./nearby-map.client";
import { useTRPC } from "@/trpc/utils";
import { useQuery } from "@tanstack/react-query";

const NearbyMap = ({ postcode }: { postcode: string }) => {
  const trpc = useTRPC();
  const nearby = useQuery(trpc.nearby.get.queryOptions({ postcode }));

  if (nearby.isLoading) {
    return (
      <div className="min-h-64 w-full grid rounded-xl bg-gray-200 animate-pulse" />
    );
  }

  if (nearby.error || !nearby.data) {
    return null;
  }

  return <NearbyMapClient nearby={nearby.data} />;
};

export default NearbyMap;
