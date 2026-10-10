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
import Link from "next/link";
import { getDatasetHistory } from "@/v2api/client";

/**
 * The names the history pages are titled with. The backend serves the keys, but
 * only the frontend knows how they should read, and only these two datasets
 * have a history page to render.
 */
const NAMES: Record<string, string> = {
  jobs: "Jobs",
  premises: "Premises",
};

/**
 * How often the history is asked for again while a sync is running, in
 * milliseconds. A sync of the collections file takes minutes, so the newest row
 * is watched until it settles rather than shown mid-flight.
 */
const WHILE_SYNCING_INTERVAL = 5_000;

const Page = ({ target }: { target: string }) => {
  const historyQuery = useQuery({
    queryKey: ["getDatasetHistory", target],
    queryFn: () => getDatasetHistory({ target }),
    refetchInterval: (query) =>
      query.state.data?.[0]?.state === "running"
        ? WHILE_SYNCING_INTERVAL
        : false,
  });

  const name = NAMES[target] ?? target;

  return (
    <div className="container my-16">
      <p className="text-muted">
        <Link href="/datasets" className="underline">
          Datasets
        </Link>
      </p>
      <h1 className="text-3xl font-semibold mt-2">{name} syncs</h1>
      <p className="text-muted mt-2">
        Every sync of the {name.toLowerCase()} dataset, newest first.
      </p>

      <div className="mt-6">
        <Table>
          <TableScrollContainer>
            <TableContent>
              <TableHeader>
                <TableColumn isRowHeader>Started</TableColumn>
                <TableColumn>Finished</TableColumn>
                <TableColumn>State</TableColumn>
                <TableColumn>Rows</TableColumn>
                <TableColumn>Source</TableColumn>
                <TableColumn>Message</TableColumn>
              </TableHeader>
              <TableBody
                renderEmptyState={() => "This dataset has never been synced."}
              >
                <TableCollection items={historyQuery.data}>
                  {(run) => (
                    <TableRow key={run.id}>
                      <TableCell className="font-mono whitespace-nowrap">
                        {timestamp(run.startedAt)}
                      </TableCell>
                      <TableCell className="font-mono whitespace-nowrap">
                        {timestamp(run.finishedAt)}
                      </TableCell>
                      <TableCell className="font-mono">{run.state}</TableCell>
                      <TableCell className="font-mono">
                        {run.rows?.toLocaleString() ?? "N/A"}
                      </TableCell>
                      <TableCell className="font-mono text-muted max-w-xs truncate">
                        {run.source}
                      </TableCell>
                      <TableCell className="text-muted max-w-sm truncate">
                        {run.message ?? ""}
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
 * A timestamp as an ISO string, or `N/A` when the sync is still going. The API
 * sends these as ISO strings already, so they are put through the date parser
 * rather than shown as they arrive.
 */
const timestamp = (value: string | null) =>
  value ? new Date(value).toISOString() : "N/A";

export default Page;
