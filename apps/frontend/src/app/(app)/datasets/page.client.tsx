"use client";

import {
  Table,
  TableBody,
  TableCell,
  TableCollection,
  TableColumn,
  TableContent,
  TableHeader,
  TableRow,
  TableScrollContainer,
} from "@heroui/react";
import { useQuery } from "@tanstack/react-query";
import { ExternalLinkIcon } from "lucide-react";
import Link from "next/link";
import { type Dataset, getDatasets } from "@/v2api/client";

/**
 * How often the datasets are asked for again while a sync is running, in
 * milliseconds. A sync of the collections file takes minutes, so the page
 * reports progress rather than waiting for the sync to end.
 */
const WHILE_SYNCING_INTERVAL = 5_000;

const Page = () => {
  const datasetsQuery = useQuery({
    queryKey: ["getDatasets"],
    queryFn: getDatasets,
    refetchInterval: (query) =>
      query.state.data?.datasets.some(
        (dataset) => dataset.sync?.state === "running",
      )
        ? WHILE_SYNCING_INTERVAL
        : false,
  });

  return (
    <div className="container my-16">
      <h1 className="text-3xl font-semibold">Datasets</h1>

      <div className="mt-6">
        <Table>
          <TableScrollContainer>
            <TableContent>
              <TableHeader>
                <TableColumn isRowHeader>Dataset</TableColumn>
                <TableColumn>CSV Size</TableColumn>
                <TableColumn>Mirror Size</TableColumn>
                <TableColumn>Last Checked</TableColumn>
                <TableColumn>Last Synced</TableColumn>
                <TableColumn>Etag</TableColumn>
                <TableColumn>Sync</TableColumn>
              </TableHeader>
              <TableBody>
                <TableCollection items={datasetsQuery.data?.datasets}>
                  {(dataset) => (
                    <TableRow key={dataset.key}>
                      <TableCell className="font-medium">
                        <Link
                          href={`/datasets/${dataset.key}`}
                          className="underline"
                        >
                          {dataset.name}
                        </Link>
                        <a
                          href={dataset.url}
                          target="_blank"
                          rel="noreferrer"
                          className="text-muted inline-flex items-center gap-1 ml-2"
                        >
                          <ExternalLinkIcon className="size-3" />
                        </a>
                      </TableCell>
                      <TableCell className="font-mono">
                        {bytes(dataset.csvSize)}
                      </TableCell>
                      <TableCell className="font-mono">
                        {bytes(dataset.dbSize)}
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
                  )}
                </TableCollection>
              </TableBody>
            </TableContent>
          </TableScrollContainer>
        </Table>
      </div>
    </div>
  );
};

/**
 * A byte count as a human-sized value, or `N/A` when it has never been
 * measured. The upstream size comes from the CSV's `content-length` header and
 * the mirror's from the size of the table on disk.
 */
const bytes = (value: number | null) => {
  if (value === null) return "N/A";

  const units = ["B", "KB", "MB", "GB"];
  let size = value;
  let unit = 0;
  while (size >= 1024 && unit < units.length - 1) {
    size /= 1024;
    unit += 1;
  }

  const digits = size >= 100 || unit === 0 ? 0 : size >= 10 ? 1 : 2;
  return `${size.toFixed(digits)} ${units[unit]}`;
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
const sync = (run: Dataset["sync"]) => {
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
