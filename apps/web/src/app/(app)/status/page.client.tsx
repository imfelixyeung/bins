"use client";

import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { StatusDataset, getStatus } from "@/v2api/client";
import { useQuery } from "@tanstack/react-query";
import Link from "next/link";
import React from "react";

/**
 * How often the status is asked for again while a sync is running, in
 * milliseconds. A sync of the collections file takes minutes, so the page
 * reports progress rather than waiting for the sync to end.
 */
const WHILE_SYNCING_INTERVAL = 5_000;

const Page = () => {
  const statusQuery = useQuery({
    queryKey: ["getStatus"],
    queryFn: getStatus,
    refetchInterval: (query) =>
      query.state.data?.datasets.some(
        (dataset) => dataset.sync?.state === "running"
      )
        ? WHILE_SYNCING_INTERVAL
        : false,
  });

  return (
    <div className="container my-16">
      <h1 className="text-3xl font-semibold">Dataset Status</h1>

      <div className="mt-6">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Dataset</TableHead>
              <TableHead>Last Checked</TableHead>
              <TableHead>Last Synced</TableHead>
              <TableHead>Etag</TableHead>
              <TableHead>Sync</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {statusQuery.data?.datasets.map((dataset) => (
              <TableRow key={dataset.key}>
                <TableCell className="font-medium">
                  <Link href={dataset.url} target="_blank">
                    {dataset.name}
                  </Link>
                </TableCell>
                <TableCell className="font-mono">
                  {timestamp(dataset.lastChecked)}
                </TableCell>
                <TableCell className="font-mono">
                  {timestamp(dataset.lastSynced)}
                </TableCell>
                <TableCell className="font-mono">
                  <code>{dataset.etag ?? "N/A"}</code>
                </TableCell>
                <TableCell className="font-mono">
                  {sync(dataset.sync)}
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>
    </div>
  );
};

/**
 * A timestamp as an ISO string, or `N/A` when the dataset has never been
 * checked. The API sends these as ISO strings already, so they are put through
 * the date parser rather than shown as they arrive, which is what the page did
 * before it read the database itself.
 */
const timestamp = (value: string | null) =>
  value ? new Date(value).toISOString() : "N/A";

/**
 * What the last sync of a dataset did, in a form that reads at a glance: a sync
 * that is still going says so, one that found nothing new says so, and a failure
 * carries the reason it stopped.
 */
const sync = (run: StatusDataset["sync"]) => {
  if (!run) return "N/A";

  switch (run.state) {
    case "running":
      return "Running...";
    case "synced":
      return run.rows === null
        ? "Synced"
        : `Synced ${run.rows.toLocaleString()} rows`;
    case "unchanged":
      return "Unchanged";
    case "failed":
      return run.message ?? "Failed";
  }
};

export default Page;
