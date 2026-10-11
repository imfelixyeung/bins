"use client";

import {
  Breadcrumbs,
  Spinner,
  Table,
  TableBody,
  TableCell,
  TableCollection,
  TableColumn,
  TableContent,
  TableHeader,
  TableRow,
  TableScrollContainer,
  Typography,
} from "@heroui/react";
import { useDatasetHistoryQuery } from "@/hooks/api";

/**
 * The names the history pages are titled with. The backend serves the keys, but
 * only the frontend knows how they should read, and only these two datasets
 * have a history page to render.
 */
const NAMES: Record<string, string> = {
  jobs: "Jobs",
  premises: "Premises",
};

const Page = ({ target }: { target: string }) => {
  const { data, isLoading } = useDatasetHistoryQuery({ target });

  const name = NAMES[target] ?? target;

  return (
    <div className="container my-16">
      <Breadcrumbs>
        <Breadcrumbs.Item href="/datasets">Datasets</Breadcrumbs.Item>
        <Breadcrumbs.Item>{name}</Breadcrumbs.Item>
      </Breadcrumbs>
      <Typography.Heading className="mt-2">{name} syncs</Typography.Heading>
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
                renderEmptyState={() => (
                  <div className="flex justify-center items-center py-6 px-2 gap-2 text-muted">
                    {isLoading ? (
                      <>
                        <Spinner /> <span>Loading</span>
                      </>
                    ) : (
                      "This dataset has never been synced."
                    )}
                  </div>
                )}
              >
                <TableCollection items={data}>
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
