import { db } from "@repo/database/src";
import { inArray } from "@repo/database/src/orm";
import { etagsTable } from "@repo/database/src/schema";
import React from "react";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import Link from "next/link";
import { connection } from "next/server";
import { unstable_cache as cache } from "next/cache";

const datasets = [
  {
    key: "jobs",
    name: "Jobs",
    url: "https://opendata.leeds.gov.uk/downloads/bins/dm_jobs.csv",
  },
  {
    key: "premises",
    name: "Premises",
    url: "https://opendata.leeds.gov.uk/downloads/bins/dm_premises.csv",
  },
];

const getData = cache(
  async () => {
    const etags = await db.query.etagsTable.findMany({
      where: inArray(
        etagsTable.url,
        datasets.map((d) => d.url)
      ),
    });

    const datasetsData = datasets.map((d) => ({
      ...d,
      etag: etags.find((e) => e.url === d.url),
    }));

    return datasetsData;
  },
  [],
  { revalidate: 5 }
);

const Page = async () => {
  await connection();

  const datasetsData = await getData();

  return (
    <div className="container my-16">
      <h1 className="text-3xl font-semibold">Dataset Status</h1>

      <div className="mt-6">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Dataset</TableHead>
              <TableHead>Last Checked</TableHead>
              <TableHead>Last Updated</TableHead>
              <TableHead>Etag</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {datasetsData.map((dataset) => (
              <TableRow key={dataset.key}>
                <TableCell className="font-medium">
                  <Link href={dataset.url} target="_blank">
                    {dataset.name}
                  </Link>
                </TableCell>
                <TableCell>
                  {dataset.etag?.checkedAt.toLocaleString()}
                </TableCell>
                <TableCell>
                  {dataset.etag?.updatedAt.toLocaleString()}
                </TableCell>
                <TableCell>{dataset.etag?.etag}</TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>
    </div>
  );
};

export default Page;
